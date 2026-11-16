#!/usr/bin/env bash
# 004-17 — election message layouts.
#
# RED-phase document check. It validates the "Election messages" section of
# doc/design/spec/wire.md and the election messages in doc/design/spec/wire.toml
# against the acceptance criteria of the ticket, and re-runs the wire-check
# binary and the wire_tables tests when the paths are not overridden.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for
# self-testing on throwaway fixtures only: when either path is overridden the
# cargo checks are skipped with a NOTE, because they always target the real
# repository.
#
# The section is located by its level-2 heading "Election messages". Each
# election message is a level-3 subsection that holds a five-column field table.
# Numbers are read from doc/design/02-constants.md at run time (C-13, C-17,
# C-32, C-33, C-34, C-37, C-100), and the hello's election epoch width, its
# type code and the type-code block of the Hello section are read from the
# landed Hello section; none is copied into this file.
#
# The author contract (headings and labels the check locates):
#
#   ## Election messages        prose: the hand-off to the hello's election epoch
#       and time-state encodings, the timers as behaviour, the winner's timeline
#       epoch rule (kept or bumped, C-213 and C-113), which message carries which,
#       the epoch comparison ("compared", "unsigned", "hello"), C-115 for the
#       reservation values.
#   ### <message title>        one per election message (title lower-cased is the
#       toml name, the word "message" optional). Its table starts with the seven
#       common control header rows in the hello's order and widths; its Message
#       type row's valid range states the code (for example "2 only"); the rows
#       tile from byte 0; a sentence "maximum size ... N bytes" equals the sum;
#       rows named Module id and Instance nonce (64 bits), Election epoch (the
#       hello's width), and where it is carried, Time state, Holder eligibility
#       (bit 0), Believed holder (64 bits), Winner timeline epoch (16 bits).
#   ### <Validity rules>        a Form / Outcome table shared by every election
#       message; a message subsection may add its own Form / Outcome table.
#   ### Open items / paragraph starting "Open item": names 005 as owner, the
#       falsifier "epic 005's election state machine needs a message or field
#       the set lacks".
#   wire.toml                   one [[message]] per election message, layer
#       "control", dscp 40, max_size equal to the markdown sum, one
#       [[message.field]] per row (name normalised, offset and width in bits),
#       rejection_reasons naming every reason the rows name.
#
# What is checked, per acceptance criterion:
#   AC 1  each message has a type code inside the block the Hello section gives
#         election messages, distinct from each other and from the hello's; the
#         common header rows in the hello's order and widths; the rows tile; the
#         stated maximum size equals the table sum and is at most C-17's bound;
#         at least one message beyond the hello.
#   AC 2  the scheme and tag rows fix the v1 value 0 and the counter row is live
#         (C-115); a sentence cites C-115.
#   AC 3  each message carries its module id and instance nonce (64 bits) and its
#         election epoch at the hello's width; across the set: a time state, the
#         hello's three values with a C-162 argument or a four-value encoding with
#         a stated mapping, cited to C-213; a holder eligibility bit 0; the
#         believed holder (64 bits); the winner's timeline epoch (16 bits), kept
#         or bumped as C-213 and C-113 say; a sentence says which message carries
#         which field.
#   AC 4  each message has at least 6 invalid forms with outcomes (shared table
#         plus its own); the ticket's list is covered (TooShort, BadMagic,
#         VersionMismatch, ForeignDomain, UnconfiguredDomain, OversizeControl,
#         BadLength, ReservedNonZero, and the election forms: election epoch,
#         holder eligibility, time state); exact reasons where the form is
#         unambiguous; election forms name a member or argue a non-rejection; no
#         outcome counts under a name outside the closed enumeration.
#   AC 5  an 'Open item' paragraph names 005 as owner with the falsifier; a
#         sentence says the election messages reuse or confirm (or differ from)
#         the hello's election epoch width and time-state encoding.
#   AC 6  wire.toml declares each election message (layer control, dscp 40 by
#         C-13, max_size, field names, offsets and widths, rejection_reasons);
#         when the paths are not overridden: `cargo run --bin wire-check` reports
#         "checks clean" and `cargo test -p hadronyche-tools --test wire_tables`
#         stays 12 passed.
#   Behaviour  multicast TTL 1 and unicast over IPv4 (C-100); CS5 (40) (C-13);
#         listen-before-claim, the election window and the reply-silence timer
#         stated with their constants' values (C-32, C-33, C-37) as behaviour; no
#         field is a timer, interval, timeout, window or period.
#   DoD   every prose paragraph of 5+ words carries an evidence tag; every table's
#         subsection has one; a paragraph starting "Open item" names an owner and
#         a falsifier.
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

ELEC_LO, ELEC_HI = 2, 15
_codes_found = False
for _title, _text in find_sub(HELLO_SUBS, r"type\s+codes"):
    for _, _, _rows in parse_tables(_text):
        for _row in _rows:
            if _row and len(_row) > 1 and re.search(r"election", _row[1], re.I):
                _nums = [int(x) for x in re.findall(r"\d+", _row[0])]
                if _nums:
                    ELEC_LO, ELEC_HI = _nums[0], _nums[-1]
                    _codes_found = True
if not _codes_found:
    fail(1, "the Hello section's Control message type codes table does not give the block for election messages")

# ---------------------------------------------------------------------------
# The Election messages section
# ---------------------------------------------------------------------------
_es = section_span(r"^election\s+messages?$", level=2)
if _es is None:
    fail(1, "wire.md has no '## Election messages' section")
EL_LINES = lines[_es[0]:_es[1]] if _es else []
EL = "\n".join(EL_LINES)
EL_PLAIN = strip_fences(EL)
EL_SUBS = subsections(EL_LINES)
_top = []
for _ln in EL_LINES:
    if re.match(r"^###\s", _ln):
        break
    _top.append(_ln)
EL_TOP = "\n".join(_top)
EL_PARAS = [t for k, t in prose_blocks(EL) if k == "p"]


def el_para(*pats):
    for t in EL_PARAS:
        if all(re.search(p, t, re.I) for p in pats):
            return t
    return None


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


MSGS = []
SHARED = []
for _title, _text in EL_SUBS:
    _tb = first_table(_text, is_field_table)
    if _tb is not None:
        MSGS.append({"title": _title, "text": _text, "fields": fields_of(_tb, 1, f"'{_title}'"),
                     "forms": form_rows(_text), "total": 0, "code": None})
    else:
        SHARED.extend(form_rows(_text))
TITLES = [m["title"] for m in MSGS]
if not MSGS:
    fail(1, "the Election messages section has no election message: a level-3 subsection with a "
         "field table is required, at least one beyond the hello")

# AC 1 and AC 2: per message, the header, the code, the size
seen_codes = {}
for m in MSGS:
    t = m["title"]
    fl = m["fields"]
    if len(fl) < 7 or [key_of(f["meaning"]) for f in fl[:7]] != HDR_KEYS:
        fail(1, f"'{t}': its table does not start with the common control header rows in the hello's "
             f"order {HDR_KEYS}")
    else:
        for a, b in zip(fl[:7], HDR):
            if (a["off"], a["bits"], a["type"]) != (b["off"], b["bits"], b["type"]):
                fail(1, f"'{t}': header row '{b['meaning'][:30]}' differs from the common control header "
                     "in offset, width or type")
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
        fail(1, f"'{t}': no sentence states its maximum size in bytes ('maximum size ... N bytes')")
    elif to_int(_ms.group(1)) != cur:
        fail(1, f"'{t}': states a maximum size of {to_int(_ms.group(1))} bytes; its table sums to {cur}")
    if K["bound"] and cur > K["bound"]:
        fail(1, f"'{t}': its maximum size {cur} bytes is above the {K['bound']}-byte control bound (C-17)")
    _tr = [f for f in fl if key_of(f["meaning"]) == "type"]
    code = None
    if _tr:
        _mc = re.search(r"\d+", _tr[0]["valid"])
        code = int(_mc.group(0)) if _mc else None
    if code is None:
        fail(1, f"'{t}': the Message type row's valid range does not state the message's type code")
    else:
        if not (ELEC_LO <= code <= ELEC_HI) or code == HELLO_TYPE:
            fail(1, f"'{t}': type code {code} is outside the block the Hello section gives election messages "
                 f"({ELEC_LO} to {ELEC_HI}), or is the hello's code")
        if code in seen_codes:
            fail(1, f"'{t}' and '{seen_codes[code]}' share type code {code}; the codes must be distinct")
        seen_codes[code] = t
    m["code"] = code
    if len(fl) >= 7:
        if not (re.search(r"\b0\b", fl[4]["valid"]) and re.search(r"only|zero|=\s*0", fl[4]["valid"], re.I)):
            fail(2, f"'{t}': the authentication scheme row does not fix the v1 value 0 (C-115)")
        if not (re.search(r"\b0\b", fl[6]["valid"]) and re.search(r"only|zero|=\s*0", fl[6]["valid"], re.I)):
            fail(2, f"'{t}': the authentication tag row does not fix the v1 value of 128 zero bits (C-115)")
        if not re.search(r"\blive\b|liveness", fl[5]["meaning"], re.I):
            fail(2, f"'{t}': the message counter row does not say it is live (C-115)")

if MSGS and not el_para(r"C-115", r"scheme|tag|reservation"):
    fail(2, "no sentence in the Election messages section cites C-115 for the v1 values of the reservation")

# AC 3, per message: the sender's module id and nonce, and the election epoch at the hello's width
for m in MSGS:
    t = m["title"]
    fl = m["fields"]
    if row_in(fl, r"module\s+id\b", 64) is None:
        fail(3, f"'{t}': no module id row of 64 bits (C-11)")
    if row_in(fl, r"instance\s+nonce\b", 64) is None:
        fail(3, f"'{t}': no instance nonce row of 64 bits (C-11)")
    ep = row_in(fl, r"election\s+epoch\b")
    if ep is None:
        fail(3, f"'{t}': no election epoch row")
    elif HELLO_EPOCH_BITS is not None and ep["bits"] != HELLO_EPOCH_BITS:
        fail(3, f"'{t}': the election epoch is {ep['bits']} bits; the hello's election epoch is "
             f"{HELLO_EPOCH_BITS} bits, and the election messages take the same width")

# AC 3, across the set
ALL_ROWS = [(m, f) for m in MSGS for f in m["fields"]]


def rows_of(pred):
    return [(m, f) for m, f in ALL_ROWS if pred(f)]


TS = rows_of(lambda f: re.match(r"\s*time\s+state\b", f["meaning"], re.I))
ELIG = rows_of(lambda f: re.match(r"\s*holder[\s-]*eligib", f["meaning"], re.I))
BELIEVED = rows_of(lambda f: re.match(r"\s*believed\s+holder\b", f["meaning"], re.I) and f["bits"] == 64)
WINNER = rows_of(lambda f: re.search(r"\bwinner", f["meaning"], re.I) and "timeline epoch" in f["meaning"].lower()
                 and f["bits"] == 16)
if MSGS:
    if not TS:
        fail(3, "no election message carries a time state: the candidate's usable time (C-213) cannot be decided")
    kinds = set()
    for m, f in TS:
        if f["bits"] != 8:
            fail(3, f"'{m['title']}': the time state is {f['bits']} bits; it is a one-byte field")
        vals = {int(x) for x in re.findall(r"(?<![\w.])(\d{1,3})\s*=", f["meaning"] + " " + f["valid"])}
        if vals == {0, 1, 2}:
            kinds.add("three")
        elif {0, 1, 2, 3} <= vals:
            kinds.add("four")
        else:
            fail(3, f"'{m['title']}': the time state row does not state 'N = meaning' for the hello's values "
                 "0, 1, 2 or for four values 0 to 3")
    if len(kinds) > 1:
        fail(3, "the election messages encode the time state two different ways; one encoding serves them all")
    if TS and not el_para(r"time\s+state", r"hello", r"C-213"):
        fail(3, "no sentence relates the election time state to the hello's time state and cites C-213")
    if "three" in kinds and not el_para(r"C-162", r"hello|three"):
        fail(3, "the hello's three-value time state is reused, but no sentence argues why three values "
             "suffice (C-162: the lapsed claim is Unlocked)")
    if "four" in kinds and not el_para(r"\bmap(s|ping|ped)?\b|\bread\s+as\b", r"hello", r"C-213"):
        fail(3, "the four-value time state has no sentence that maps it to the hello's three values")
    if not ELIG:
        fail(3, "no election message carries holder eligibility (a bit consistent with the hello's capability bit 0)")
    for m, f in ELIG:
        if "bit 0" not in (f["meaning"] + " " + f["valid"]).lower():
            fail(3, f"'{m['title']}': holder eligibility does not name bit 0, the hello's holder-eligible bit")
        if f["bits"] not in (1, 8):
            fail(3, f"'{m['title']}': holder eligibility is {f['bits']} bits; a bit or a one-byte field")
    if not BELIEVED:
        fail(3, "no election message carries the believed holder (module id, 64 bits)")
    if not WINNER:
        fail(3, "no election message carries the winner's timeline epoch (16 bits, C-10)")
    if not el_para(r"timeline\s+epoch", r"C-213", r"kept|keeps", r"bump", r"C-113", r"lowest"):
        fail(3, "no sentence restates as fact that the winner's timeline epoch is kept when the winner has "
             "usable time (C-213) and bumped when no candidate has usable time, the lowest module id winning (C-113)")
    if not el_para(r"election\s+epoch", r"compared", r"unsigned", r"hello"):
        fail(3, "no sentence says that the election epoch is compared unsigned, as in the hello")
    if not any(re.search(r"carr", p, re.I) and all(re.search(re.escape(t), p, re.I) for t in TITLES)
               for p in EL_PARAS):
        fail(3, "no sentence says which election message carries which field: a paragraph with 'carries' "
             "must name every election message")

# AC 4: invalid forms with outcomes
EXACT = {"short": "TooShort", "magic": "BadMagic", "version": "VersionMismatch", "domain0": "UnconfiguredDomain",
         "foreign": "ForeignDomain", "oversize": "OversizeControl", "length": "BadLength",
         "reserved": "ReservedNonZero"}
ELECTION_CATS = ("epoch", "eligibility", "timestate")
NON_REJECTION = re.compile(r"not\s+a\s+(?:parse\s+)?rejection|never\s+a\s+rejection|is\s+not\s+counted|"
                           r"cannot\s+occur|ignored", re.I)


def classify(form):
    f = form.lower()
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
    if "epoch" in f:
        return "epoch"
    if "eligib" in f:
        return "eligibility"
    if re.search(r"time[\s-]*state", f):
        return "timestate"
    if re.search(r"\bsize\b|length", f):
        return "length"
    if re.search(r"reserved|non-?zero|scheme|\btag\b", f):
        return "reserved"
    return None


def named_reasons(outcome):
    found = []
    for m in re.finditer(r"\b[A-Za-z]+\b", outcome):
        w = m.group(0)
        if w in MEMBERS:
            before = outcome[max(0, m.start() - 24):m.start()]
            if re.search(r"\b(not|never|nor|no|neither)\b[^|;.]*$", before, re.I):
                continue
            found.append(w)
    return found


CATS_SEEN = set()


def check_form(form, outcome, where):
    for mm in re.findall(r"count\s+under\s+`?([A-Za-z]+)`?", outcome):
        if mm not in MEMBERS:
            fail(4, f"{where}: the form '{form[:50]}' counts under '{mm}', which is not in the closed enumeration")
    for name in CAMEL.findall(outcome):
        if name not in MEMBERS and not re.search(r"\b(not|never|nor|no)\b[^|;.]*" + re.escape(name), outcome):
            fail(4, f"{where}: the outcome names '{name}', which is not in the closed enumeration: no invented reason")
    cat = classify(form)
    if cat:
        CATS_SEEN.add(cat)
    named = named_reasons(outcome)
    if cat in EXACT:
        if not re.search(r"\b" + EXACT[cat] + r"\b", outcome):
            fail(4, f"{where}: the form '{form[:50]}' must be counted under {EXACT[cat]}: {outcome[:70]}")
        if cat == "domain0" and "ForeignDomain" in named:
            fail(4, f"{where}: domain 0 is mapped to ForeignDomain; it is ignored and counted as UnconfiguredDomain")
        if cat == "version" and not re.search(r"\bevent\b", outcome, re.I):
            fail(4, f"{where}: the version mismatch row does not say that it raises a named event")
    elif cat in ELECTION_CATS:
        if not (named or NON_REJECTION.search(outcome)):
            fail(4, f"{where}: the election form '{form[:50]}' needs a member reason or an argued non-rejection")
    elif not (named or NON_REJECTION.search(outcome)):
        fail(4, f"{where}: the form '{form[:50]}' names no member of the closed enumeration and argues no non-rejection")


for m in MSGS:
    total_forms = len(SHARED) + len(m["forms"])
    if total_forms < 6:
        fail(4, f"'{m['title']}' has {total_forms} invalid forms with outcomes (the shared table and its own); "
             "at least 6 are required")
    for f_, o_ in m["forms"]:
        check_form(f_, o_, f"'{m['title']}'")
for f_, o_ in SHARED:
    check_form(f_, o_, "the shared Validity rules")
if not SHARED and not any(m["forms"] for m in MSGS):
    fail(4, "no invalid-form table with 'Form' and 'Outcome' columns in the Election messages section")
for _c in ("short", "magic", "version", "domain0", "foreign", "oversize", "length", "reserved") + ELECTION_CATS:
    if _c not in CATS_SEEN:
        fail(4, f"no invalid form covers the '{_c}' case of the ticket's list")

# AC 5: the open item, and the hand-off to the hello
_opens = [t for t in EL_PARAS if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(t), re.I)]
if not any(re.search(r"owner[^.;]*?\b005\b", t, re.I) and re.search(r"falsifier", t, re.I)
           and re.search(r"needs\s+a\s+message\s+or\s+field", t, re.I) for t in _opens):
    fail(5, "no paragraph starting 'Open item' names epic 005 as the confirming owner (the 'Owner' label) and the falsifier "
         "'epic 005's election state machine needs a message or field the set lacks'")
if not el_para(r"hello", r"election\s+epoch", r"time\s+state", r"reuse|confirm|differ"):
    fail(5, "no sentence says that the election messages reuse or confirm the hello's election epoch width and "
         "time-state encoding, or states the difference")

# Behaviour restated as fact, with the constants' values; no timer as a field
if not (re.search(r"TTL\s+1\b", EL_PLAIN) and "C-100" in EL_PLAIN and re.search(r"unicast", EL_PLAIN, re.I)
        and re.search(r"IPv4", EL_PLAIN)):
    fail("Behaviour", "multicast with TTL 1 and unicast over IPv4 (C-100) is not restated as fact")
if K["dscp_control"] is not None and not (re.search(r"CS5\s*\(%d\)" % K["dscp_control"], EL_PLAIN) and "C-13" in EL_PLAIN):
    fail("Behaviour", "CS5 (40) with C-13 is not restated as fact")
for _key, _cid in (("lbc", "C-32"), ("window", "C-33"), ("silence", "C-37")):
    _d = DUR[_key]
    if _d is None:
        continue
    if not (re.search(r"(?<![\d.])" + re.escape(_d[0]) + r"\s*" + _d[1] + r"\b", EL_PLAIN) and _cid in EL_PLAIN):
        fail("Behaviour", f"{_cid} ({_d[0]} {_d[1]}) is not restated as behaviour in the section")
if "C-34" in EL_PLAIN and K["peers_max"] is not None and not re.search(r"\b%d\b" % K["peers_max"], EL_PLAIN):
    fail("Behaviour", "C-34 is cited but its bound is not restated")
TIMER = re.compile(r"timer|interval|timeout|time[\s-]*out|listen|window|silence|period|expir|deadline|"
                   r"backoff|delay|\bms\b|seconds?\b", re.I)
for m in MSGS:
    for f in m["fields"]:
        if TIMER.search(md_name(f["meaning"])):
            fail("Behaviour", f"'{m['title']}': the field '{md_name(f['meaning'])}' is a timer; a timer is behaviour, "
                 "not a field")

# DoD: evidence tags, and owners and falsifiers on open items
for title, text in [("(section)", EL_TOP)] + EL_SUBS:
    has_tag_here = has_evidence(text)
    for kind, body in prose_blocks(text):
        if kind == "p":
            words = len(re.findall(r"[A-Za-z]{2,}", re.sub(r"`[^`]*`", "", body)))
            if words >= 5 and not has_evidence(body):
                fail("DoD", f"a prose paragraph of '{title}' carries no evidence tag: {body.splitlines()[0][:80]}")
            if re.match(r"\s*\*{0,2}\s*open\s+items?\b", clean_start(body), re.I):
                if not re.search(r"owner", body, re.I) or not re.search(r"falsifier", body, re.I):
                    fail("DoD", f"an open item in '{title}' names no owner or no falsifier: "
                         f"{body.splitlines()[0][:80]}")
        elif kind == "t" and not has_tag_here:
            fail("DoD", f"a table of '{title}' follows no evidence-tagged paragraph")

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
# AC 6: wire.toml declares each election message
# ---------------------------------------------------------------------------
def tname(text):
    return re.sub(r"\s+messages?$", "", norm_name(text))


tables_text = read_text(tables_path)
toml = None
if tables_text is None:
    fail(6, f"{tables_path} does not exist")
else:
    try:
        toml = parse_toml(tables_text)
    except TomlError as error:
        fail(6, f"{tables_path} does not parse: {error}")
msgs_t = toml.get("message", []) if isinstance(toml, dict) else []
if not isinstance(msgs_t, list):
    msgs_t = []
PRIM = {"u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128"}
for m in MSGS:
    t = m["title"]
    cands = [x for x in msgs_t if isinstance(x, dict) and tname(str(x.get("name", ""))) == tname(t)]
    if not cands:
        fail(6, f"wire.toml declares no [[message]] for the election message '{t}'")
        continue
    T = cands[0]
    if T.get("layer") != "control":
        fail(6, f"toml message '{T.get('name')}' has layer '{T.get('layer')}', not 'control'")
    if K["dscp_control"] is not None and T.get("dscp") != K["dscp_control"]:
        fail(6, f"toml message '{T.get('name')}' has dscp {T.get('dscp')}; C-13 fixes CS5 ({K['dscp_control']})")
    if T.get("max_size") != m["total"]:
        fail(6, f"toml message '{T.get('name')}' has max_size {T.get('max_size')}; the markdown table sums to {m['total']}")
    if K["bound"] and isinstance(T.get("max_size"), int) and T["max_size"] > K["bound"]:
        fail(6, f"toml message '{T.get('name')}' max_size {T['max_size']} exceeds the {K['bound']}-byte bound (C-17)")
    tf = [x for x in (T.get("field") or []) if isinstance(x, dict)]
    if len(tf) != len(m["fields"]):
        fail(6, f"toml message '{T.get('name')}' has {len(tf)} fields; the markdown table has {len(m['fields'])} rows")
    total_bits = 0
    for mf, tfl in zip(m["fields"], tf):
        nm = md_name(mf["meaning"])
        if norm_name(nm) != norm_name(str(tfl.get("name", ""))):
            fail(6, f"toml field '{tfl.get('name')}' is not named after the markdown row '{nm}' "
                 "(names are compared lower case, spaces and underscores equivalent)")
        if tfl.get("offset") != mf["off"] * 8:
            fail(6, f"toml field {tfl.get('name')}: offset {tfl.get('offset')} bits, markdown byte {mf['off']} "
                 f"(= {mf['off'] * 8} bits)")
        if tfl.get("width") != mf["bits"]:
            fail(6, f"toml field {tfl.get('name')}: width {tfl.get('width')} bits; markdown {mf['bits']} bits")
        if mf["maxc"] is not None and tfl.get("max_count") != mf["maxc"]:
            fail(6, f"toml field {tfl.get('name')}: max_count {tfl.get('max_count')}; markdown says max {mf['maxc']}")
        if mf["type"] in PRIM and tfl.get("type") != mf["type"]:
            fail(6, f"toml field {tfl.get('name')}: type {tfl.get('type')}; markdown {mf['type']}")
        if TIMER.search(str(tfl.get("name", ""))):
            fail("Behaviour", f"toml field '{tfl.get('name')}' is named like a timer")
        total_bits += (tfl.get("width", 0) or 0) * (tfl.get("max_count") or 1)
    if isinstance(T.get("max_size"), int) and total_bits != T["max_size"] * 8:
        fail(6, f"the toml fields of '{T.get('name')}' compute {total_bits // 8} bytes; max_size is {T.get('max_size')}")
    need = set()
    for _, o_ in SHARED + m["forms"]:
        need.update(named_reasons(o_))
    have = set(T.get("rejection_reasons") or [])
    for r in sorted(need - have):
        fail(6, f"toml message '{T.get('name')}' rejection_reasons does not list '{r}', which its invalid-form rows name")
    for r in sorted(have - set(MEMBERS)):
        fail(6, f"toml message '{T.get('name')}' rejection_reasons names '{r}', not in the closed enumeration")

# ---------------------------------------------------------------------------
# AC 6 (repository level): the wire-check binary and the wire_tables tests
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

print("OK: election messages with type codes, common headers, maximum sizes, time state and holder "
      "eligibility, winner's timeline epoch, invalid forms, open item, toml agreement and DoD tags present")
PY
