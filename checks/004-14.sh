#!/usr/bin/env bash
# 004-14 — media packet size budget at 1 472 and 1 280 bytes.
#
# RED-phase document check. It validates the "Size budget" section of
# doc/design/spec/wire.md, the pointers the landed sections carry to it, and the
# machine-readable totals of doc/design/spec/wire.toml against every acceptance
# criterion of the ticket. It does not run the other checks/004-*.sh: the
# per-change suite runs every file in checks/, so a sibling loop here would be
# quadratic.
#
# WIRE_MD / WIRE_TOML may override the paths. The overrides are for self-testing
# on throwaway fixtures only: when either path is overridden the repository-level
# check (cargo run --bin wire-check, used only when wire.toml carries no budget
# table) is skipped with a NOTE, because it always targets the real repository.
# The constants are always read from doc/design/02-constants.md.
#
# Every number the check uses is computed at run time from the landed field
# tables (wire.toml [[message]] / [[message.field]] and the constants table):
# nothing about packet, slice, announcement, extension, checksum or content
# sizes is pinned here. The only literals are the path arithmetic the ticket
# states (Ethernet MTU 1 500, IPv4 header 20, UDP header 8).
#
# The author's form, which the check reads (all by content, not by position):
#
#   Section     a level-2 heading "Size budget" in wire.md; its level-3
#               subsections are free. Its paragraphs carry evidence tags.
#   Budget      a Markdown table with a row whose first cell starts with
#               "Total". Every numeric column of such a table is one case. The
#               case is named by the table's caption (the lines above it) or its
#               column header: "fallback" or 1 280 / 1 252 -> the fallback case
#               (payload bound 1 252); "worst", "maximum" or "all optional" ->
#               worst case (1 472); "default" or "mandatory" -> default case
#               (1 472). Rows are classified by label: "packet header", "slice
#               header", "announce..." (every announcement row sums to the
#               container: count byte 1 plus 12 per announcement, at most 49),
#               "content-message header", "audio", "checksum", "measurement
#               extension", "Spare" (or margin/headroom), "Total"; controller
#               rows are allowed; any other label fails. Required per case: a
#               packet header, slice header, announcement, content-message
#               header, audio and checksum row, and a spare row. Values are
#               parsed from the cell (thousands with spaces; a "= ..." tail is
#               taken; a parenthesis is ignored; a dash or "omitted" is 0).
#   Cases       at least one default case (announcements = count byte only, no
#               extension); at least one worst case with announcements at
#               maximum and the extension (total 1 255); at least one worst
#               case with announcements at maximum and no extension; at least
#               one fallback case.
#   Allowance   a table with a header cell containing "allow": columns named
#               (any header containing) bound/cap/payload, audio, count/header,
#               checksum, allow; one row labelled 1 472 and one labelled
#               1 280 / fallback.
#   Verdict     a paragraph (or table row) containing "verdict". "C-137 holds",
#               "holds", "not falsified" or "confirmed" make a holds verdict;
#               "falsified" (not "not falsified") makes a falsified verdict.
#   Omission    paragraphs or rows containing "omit" name announcements,
#               measurement extension and controller messages, and say the
#               order (priority) in which they are omitted.
#   Toml        optional: a [budget] or [[budget...]] table in wire.toml whose
#               integers are totals, spares or components of wire.md. If absent,
#               cargo run --bin wire-check must print the default and fallback
#               totals (skipped with a NOTE under overrides).
#
# What is checked, per acceptance criterion:
#
#   AC 1  The 1 472 default case: a budget table whose rows are the mandatory
#         parts (packet header 16, slice header 22, announcement count byte 1,
#         content-message header 3, audio 1 152 = 6 x 48 x 4 from C-04, C-02,
#         C-03, checksum 4), with total equal to their sum, spare equal to
#         1 472 minus total, total at most 1 472. Every component equals the
#         size the landed toml declares for it. The section cites C-100, C-06
#         and C-137.
#   AC 2  Worst case at 1 472 with all optional parts at maximum (announcements
#         49, extension 9): total and spare shown, and the same without the
#         extension (announcements 49, extension 0). Arithmetic checked.
#   AC 3  Fallback at 1 280 (payload 1 252): 6 channels (1 152), every mandatory
#         header and the checksum present, the non-audio bytes at most 100 (C-137),
#         the total at most 1 252 and spare = 1 252 minus total. The omission
#         rule names announcements, the measurement extension and controller
#         messages, and states a priority. A fallback that omits an announcement
#         or the extension must say it amends or scopes the landed sender
#         obligations (C-44, 004-10). The checksum is never omitted.
#   AC 4  The verdict is consistent with the computed mandatory headers plus
#         checksum (46 bytes from the tables) against 100: a fit means the verdict
#         must hold and must not say falsified; a non-fit means falsified, with
#         "5 channels" and C-04's constants value 5.
#   AC 5  Every total in wire.md equals the sum of its rows, and every component
#         equals the toml-computed size; the toml budget values (if any) are
#         totals, spares or components of wire.md and include every case total;
#         otherwise the 004-02 checker must print the totals.
#   AC 6  The controller allowance table: allowance = bound - audio - counted
#         headers - checksum at both path sizes, with audio 1 152, checksum from
#         toml, counted headers from the computed mandatory parts plus an
#         allowed announcement and extension count, and bound 100, 1 252 or
#         1 472. The section says the allowance is an upper bound (most a source
#         may share), that a source may always use a second stream (C-181), and
#         that a bundle source's audio member counts no controller messages
#         (C-205). A sentence names transport state and sparse entries with the
#         toml sizes (30 and 8) and a count of sparse entries that fits beside a
#         transport state inside one of the allowances.
#   Pointers  No "open item owned by 004-14" and no "But see ... Budget" pointer
#         remains anywhere; no unresolved pointer to 004-14 (Budget) outside the
#         section; the header-cap open item of Content messages, Budget is closed
#         ("closed", "resolved" or "answered"); each landed sentence that the
#         budget contradicts (the four "fit the cap" statements and the
#         extension-omission statement) is either removed or its paragraph cites
#         the Size budget section; a "provisional" line about the shared-packet
#         cap row is not left unless the section says why it stays
#         ("stays provisional" and "because").
#   DoD   every prose paragraph of 5 or more words in the section carries an
#         evidence tag; a paragraph beginning "Open item" names an owner and a
#         falsifier.
#
# Compatibility: the check does not pin the old pointer sentences or the word
# "provisional" outside the rules above; the sibling checks (004-09 to 004-13)
# are run by their own testers on fixtures, not from here.
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
TABLES="${WIRE_TOML:-doc/design/spec/wire.toml}"
CONSTANTS="doc/design/02-constants.md"
python3 - "$WIRE" "$TABLES" "$CONSTANTS" <<'PY'
import os
import re
import subprocess
import sys

wire_path, toml_path, const_path = sys.argv[1], sys.argv[2], sys.argv[3]
overridden = bool(os.environ.get("WIRE_MD") or os.environ.get("WIRE_TOML"))

fails = []


def fail(ac, msg):
    fails.append((ac, msg))


def read(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return None


wire = read(wire_path)
toml_src = read(toml_path)
const_src = read(const_path)
for label, text, path in (("wire.md", wire, wire_path), ("wire.toml", toml_src, toml_path),
                          ("02-constants.md", const_src, const_path)):
    if text is None:
        print(f"FAIL setup: cannot read {label} at {path}")
        sys.exit(1)

NUM_RE = re.compile(r"\d{1,3}(?:[   ]\d{3})+|\d+")
WORD_NUM = {"zero": 0, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
            "seven": 7, "eight": 8, "nine": 9, "ten": 10}


def to_int(s):
    return int(re.sub(r"[   ,]", "", s))


def first_int(text):
    m = NUM_RE.search(text or "")
    return to_int(m.group(0)) if m else None


def norm(s):
    return re.sub(r"\s+", " ", s)


def split_row(line):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|"):
        s = s[:-1]
    return [c.strip().replace("\\|", "|") for c in re.split(r"(?<!\\)\|", s)]


# ---------------------------------------------------------------------------
# Constants (read, never edited). Values come from the constants table.
# ---------------------------------------------------------------------------
def const_value(cid):
    for line in const_src.splitlines():
        if not line.lstrip().startswith("|"):
            continue
        cells = split_row(line)
        if len(cells) >= 3 and cells[0] == cid:
            return cells[2]
    return None


CH = first_int(const_value("C-04"))        # channels per stream at defaults (6)
FRAMES = first_int(const_value("C-02"))    # frames per default slice (48)
SAMPLE = first_int(const_value("C-03"))    # bytes per sample (4)
FB_MTU = first_int(const_value("C-06"))    # path-MTU fallback (1 280)
FB_PAY = first_int(const_value("C-17"))    # control bound, UDP payload (1 252)
m137 = re.search(r"at most\s+(\d+)\s+bytes", const_value("C-137") or "")
CAP = int(m137.group(1)) if m137 else None  # header cap of C-137 (100)
ETH_MTU, IP_HDR, UDP_HDR = 1500, 20, 8      # ticket 004-14, C-100 arithmetic
PAY_DEF = ETH_MTU - IP_HDR - UDP_HDR        # 1 472
PAY_FB = FB_MTU - IP_HDR - UDP_HDR          # must equal the C-17 bound
if None in (CH, FRAMES, SAMPLE, FB_MTU, FB_PAY, CAP):
    print("FAIL setup: a constant the budget needs (C-02, C-03, C-04, C-06, C-17, C-137) is unreadable")
    sys.exit(1)
if PAY_FB != FB_PAY:
    print(f"FAIL setup: C-06 minus IPv4 and UDP headers is {PAY_FB}, but C-17 states {FB_PAY}")
    sys.exit(1)
AUDIO = CH * FRAMES * SAMPLE


# ---------------------------------------------------------------------------
# A small TOML reader for the subset wire.toml uses (Python 3.10 has no
# tomllib; the sibling checks use the same kind of reader).
# ---------------------------------------------------------------------------
def strip_toml_comment(line):
    out = []
    in_str = False
    i = 0
    while i < len(line):
        c = line[i]
        if in_str:
            out.append(c)
            if c == "\\" and i + 1 < len(line):
                out.append(line[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
            out.append(c)
        elif c == "#":
            break
        else:
            out.append(c)
        i += 1
    return "".join(out)


def parse_value(s):
    s = s.strip()
    if s.startswith('"'):
        m = re.match(r'"((?:[^"\\]|\\.)*)"', s)
        return m.group(1).replace('\\"', '"').replace("\\\\", "\\") if m else None
    if s.startswith("["):
        items = []
        for m in re.finditer(r'"((?:[^"\\]|\\.)*)"|(-?0x[0-9a-fA-F_]+|-?\d[\d_]*)', s):
            if m.group(1) is not None:
                items.append(m.group(1))
            else:
                items.append(int(m.group(2).replace("_", ""), 0))
        return items
    if s in ("true", "false"):
        return s == "true"
    t = s.replace("_", "")
    try:
        return int(t, 0)
    except ValueError:
        try:
            return int(t, 10)
        except ValueError:
            return s


def parse_toml(src):
    messages = []          # [[message]] dicts; "fields" holds [[message.field]] dicts
    others = {}            # every other table name -> dict, or list of dicts
    target = None
    for raw in src.splitlines():
        line = strip_toml_comment(raw).strip()
        if not line:
            continue
        if line.startswith("[["):
            name = line[2:line.index("]]")].strip()
            if name == "message":
                target = {"name": None, "fields": []}
                messages.append(target)
            elif name == "message.field":
                if not messages:
                    raise ValueError("a [[message.field]] precedes any [[message]]")
                target = {}
                messages[-1]["fields"].append(target)
            else:
                target = {}
                others.setdefault(name, [])
                if not isinstance(others[name], list):
                    raise ValueError(f"table {name} is both an array and a table")
                others[name].append(target)
            continue
        if line.startswith("["):
            name = line[1:line.index("]")].strip()
            target = {}
            others[name] = target
            continue
        if "=" in line and target is not None:
            key, value = line.split("=", 1)
            target[key.strip()] = parse_value(value)
    return messages, others


try:
    MESSAGES, OTHERS = parse_toml(toml_src)
except (ValueError, IndexError) as error:
    print(f"FAIL setup: wire.toml does not parse: {error}")
    sys.exit(1)

BY_NAME = {m.get("name"): m for m in MESSAGES if m.get("name")}


def msg(name):
    m = BY_NAME.get(name)
    if m is None:
        fail("AC 5", f"wire.toml has no [[message]] named {name!r}")
    return m


def field(m, name):
    for f in (m or {}).get("fields", []):
        if f.get("name") == name:
            return f
    return None


def fbytes(f):
    return f["width"] // 8 if f and isinstance(f.get("width"), int) else None


mp = msg("media packet")
ann_count_f = field(mp, "announcement_count")
ann_entry_f = field(mp, "announcement_entry")
ck = msg("slice checksum")
cw_m = msg("content-message header")
audio_m = msg("audio message")
transport_m = msg("transport state message")
sparse_entry_m = msg("sparse keyed entry")
sparse_m = msg("sparse keyed controller message")
sparse_entry_f = field(sparse_m, "entry")
audio_sample_f = field(audio_m, "sample")

COMP = {}
if mp and ann_count_f and ann_entry_f:
    # packet header: the fields tiling bits [0, 128); slice header: [128, announcement count)
    hdr_fields = [f for f in mp["fields"] if f.get("offset", 0) + f.get("width", 0) <= 128]
    COMP["pkt"] = sum(f["width"] for f in hdr_fields) // 8
    COMP["slice"] = sum(f["width"] for f in mp["fields"]
                        if 128 <= f.get("offset", -1) < ann_count_f["offset"]) // 8
    COMP["ann_count"] = fbytes(ann_count_f)
    COMP["ann_entry"] = fbytes(ann_entry_f)
    COMP["ann_max_n"] = ann_entry_f.get("max_count")
    ext_start = ann_entry_f["offset"] + ann_entry_f["width"] * ann_entry_f["max_count"]
    COMP["ext"] = sum(f["width"] for f in mp["fields"] if f.get("offset", -1) >= ext_start) // 8
    COMP["ext_start"] = ext_start
if ck:
    COMP["cksum"] = ck.get("max_size")
    cf = field(ck, "checksum")
    if fbytes(cf) != COMP.get("cksum"):
        fail("AC 5", "wire.toml 'slice checksum' max_size does not equal its field width")
if cw_m:
    COMP["cw"] = cw_m.get("max_size")
if transport_m:
    COMP["transport"] = transport_m.get("max_size")
if sparse_entry_m:
    COMP["sp_entry"] = sparse_entry_m.get("max_size")
if sparse_m and sparse_entry_f and COMP.get("sp_entry") is not None:
    COMP["sp_hdr"] = sparse_m.get("max_size") - COMP["sp_entry"] * sparse_entry_f.get("max_count", 0)
if audio_m and audio_sample_f:
    COMP["sample_bytes"] = fbytes(audio_sample_f)

need = ("pkt", "slice", "ann_count", "ann_entry", "ann_max_n", "ext", "cksum", "cw",
        "transport", "sp_entry", "sp_hdr", "sample_bytes")
missing_comp = [k for k in need if k not in COMP]
if missing_comp:
    print("FAIL AC 5: wire.toml does not declare what the budget needs: " + ", ".join(missing_comp))
    sys.exit(1)

if True:
    if COMP["sample_bytes"] != SAMPLE:
        fail("AC 5", f"wire.toml audio sample is {COMP['sample_bytes']} bytes, C-03 says {SAMPLE}")
    if COMP["ann_count"] != 1:
        fail("AC 5", "wire.toml announcement count is not one byte")
    if COMP["ext_start"] != COMP["pkt"] * 8 + COMP["slice"] * 8 + COMP["ann_count"] * 8 + \
            COMP["ann_entry"] * 8 * COMP["ann_max_n"]:
        fail("AC 5", "the measurement extension does not start where the maximal container ends")
    if mp and mp.get("max_size") != COMP["pkt"] + COMP["slice"] + COMP["ann_count"] + \
            COMP["ann_entry"] * COMP["ann_max_n"] + COMP["ext"]:
        fail("AC 5", "wire.toml 'media packet' max_size is not header + slice + container max + extension")

PKT = COMP.get("pkt")
SLICE = COMP.get("slice")
ANN_COUNT = COMP.get("ann_count")
ENTRY = COMP.get("ann_entry")
ENTRY_MAX = COMP.get("ann_max_n")
ANN_MAX = ANN_COUNT + ENTRY * ENTRY_MAX
EXT = COMP.get("ext")
CKSUM = COMP.get("cksum")
CW = COMP.get("cw")
TRANSPORT = COMP.get("transport")
SP_ENTRY = COMP.get("sp_entry")
SP_HDR = COMP.get("sp_hdr")
MANDATORY = PKT + SLICE + ANN_COUNT + CW + CKSUM
MAX_NON_AUDIO = MANDATORY + ENTRY * ENTRY_MAX + EXT
ALLOWED_ANN_BYTES = {ANN_COUNT + ENTRY * k for k in range(ENTRY_MAX + 1)}
ALLOWED_COUNTED = {PKT + SLICE + ANN_COUNT + CW + (ENTRY * k) + e
                   for k in range(ENTRY_MAX + 1) for e in (0, EXT)}

# ---------------------------------------------------------------------------
# Markdown: the Size budget section, its tables, its paragraphs.
# ---------------------------------------------------------------------------
lines = wire.splitlines()


def fence_mask(ls):
    mask, inside = [], False
    for line in ls:
        if re.match(r"^\s*```", line):
            inside = not inside
            mask.append(True)
            continue
        mask.append(inside)
    return mask


WIRE_FENCE = fence_mask(lines)
headings = []
for i, line in enumerate(lines):
    if WIRE_FENCE[i]:
        continue
    m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
    if m:
        headings.append((i, len(m.group(1)), m.group(2).strip()))

sec_start = sec_end = None
for k, (idx, level, title) in enumerate(headings):
    if level == 2 and re.fullmatch(r"size budget", title, re.I):
        sec_start = idx
        sec_end = next((j for (j, lv, _) in headings[k + 1:] if lv <= 2), len(lines))
        break

if sec_start is None:
    fail("AC 1", "wire.md has no level-2 heading 'Size budget'")
    sec_lines = []
else:
    sec_lines = lines[sec_start:sec_end]
sec_fence = fence_mask(sec_lines)
glob_lines = lines[:sec_start] + lines[sec_end:] if sec_start is not None else list(lines)
sec_text = norm(" ".join(l for l, f in zip(sec_lines, sec_fence) if not f))


def is_table_line(line):
    return line.lstrip().startswith("|")


def caption_above(ls, fm, i):
    got = []
    k = i - 1
    while k >= 0 and i - k <= 5:
        s = ls[k].strip()
        if s and not is_table_line(ls[k]) and not fm[k]:
            got.append(s.lstrip("#* ").strip())
            if len(got) == 2:
                break
        k -= 1
    return " ".join(reversed(got))


def section_tables(ls, fm):
    out = []
    i = 0
    while i < len(ls):
        if fm[i] or not is_table_line(ls[i]):
            i += 1
            continue
        j = i
        while j < len(ls) and not fm[j] and is_table_line(ls[j]):
            j += 1
        block = ls[i:j]
        if len(block) >= 2 and "-" in block[1] and re.match(r"^\s*\|?[\s:|-]+\|?\s*$", block[1]):
            out.append({"start": i, "end": j, "header": split_row(block[0]),
                        "rows": [split_row(r) for r in block[2:]],
                        "caption": caption_above(ls, fm, i)})
        i = j
    return out


TABLES_SEC = section_tables(sec_lines, sec_fence)


def label_of(cell):
    return re.sub(r"[*_`]", "", cell or "").strip().lower()


DASHES = {"", "—", "–", "-", "none", "omitted", "absent", "n/a", "not sent"}


def parse_num(cell):
    raw = (cell or "").strip()
    low = re.sub(r"[*_`]", "", raw).strip().lower()
    if low in DASHES:
        return 0
    s = raw.replace("*", "")
    if "=" in s:
        s = s.split("=")[-1]
    s = re.sub(r"\([^)]*\)", "", s)
    m = NUM_RE.search(s)
    if not m:
        return None
    value = to_int(m.group(0))
    if re.search(r"[−]\s*\d|(?<![\w])-\s*\d|deficit", s, re.I):
        value = -value
    return value


def row_kind(lab):
    if lab.startswith("total"):
        return "total"
    if re.search(r"\b(spare|margin|headroom|slack|deficit)\b", lab):
        return "spare"
    if "packet header" in lab:
        return "pkt"
    if "slice header" in lab:
        return "slice"
    if "announce" in lab:
        return "ann"
    if re.search(r"content[- ]message|content header", lab):
        return "content"
    if "audio" in lab:
        return "audio"
    if "checksum" in lab or re.search(r"\bcrc\b", lab):
        return "cksum"
    if "measurement" in lab or "extension" in lab:
        return "ext"
    if re.search(r"controller|dense|sparse|transport", lab):
        return "ctrl"
    return "other"


def classify_case(text):
    if re.search(r"fallback|\b1\s?280\b|\b1\s?252\b", text, re.I):
        return "fb"
    if re.search(r"worst|maximum|maximal|all optional", text, re.I):
        return "worst"
    if re.search(r"default|mandatory|\b1\s?472\b", text, re.I):
        return "default"
    return None


CASES = []
for t in TABLES_SEC:
    rows = t["rows"]
    hdr = t["header"]
    totals = [r for r in rows if r and label_of(r[0]).startswith("total")]
    # A budget table has a Total row, a Spare row and an audio row. Any other table in the
    # section (a wire-size table of 004-15, say) is not a budget table and AC 1 to 3 ignore it.
    present = {row_kind(label_of(r[0])) for r in rows if r and label_of(r[0])}
    if not totals or not {"spare", "audio"} <= present:
        continue
    for c in range(1, len(hdr)):
        if parse_num(totals[0][c] if c < len(totals[0]) else None) is None:
            continue
        ctext = t["caption"] + " " + hdr[c]
        kind = classify_case(ctext)
        label = f"{t['caption'][:60]!r} column {hdr[c]!r}"
        if kind is None:
            fail("AC 1", f"budget column {label} is not named as default, worst case or fallback")
            continue
        vals = {}
        spare = None
        total = parse_num(totals[0][c])
        bad = False
        for r in rows:
            if not r or len(r) <= c or not label_of(r[0]):
                continue
            k = row_kind(label_of(r[0]))
            v = parse_num(r[c])
            if v is None:
                fail("AC 1", f"budget column {label}: row {r[0]!r} has no number")
                bad = True
                continue
            if k == "total":
                continue
            if k == "spare":
                spare = v
                continue
            if k == "other":
                fail("AC 1", f"budget column {label}: row {r[0]!r} is not a recognised budget part")
                bad = True
                continue
            vals.setdefault(k, []).append(v)
        if bad:
            continue
        if spare is None:
            fail("AC 1", f"budget column {label} has no spare row")
            continue
        comp_total = sum(sum(v) for k, v in vals.items() if k != "spare")
        if comp_total != total:
            fail("AC 5", f"budget column {label}: the parts sum to {comp_total}, its total row says {total}")
        for need_k in ("pkt", "slice", "ann", "content", "audio", "cksum"):
            if need_k not in vals:
                fail("AC 1", f"budget column {label} has no {need_k} row")
        if "pkt" in vals and PKT is not None and vals["pkt"] != [PKT]:
            fail("AC 1", f"budget column {label}: packet header {vals['pkt']} != {PKT} (toml)")
        if "slice" in vals and SLICE is not None and vals["slice"] != [SLICE]:
            fail("AC 1", f"budget column {label}: slice header {vals['slice']} != {SLICE} (toml)")
        if "content" in vals and CW is not None and vals["content"] != [CW]:
            fail("AC 1", f"budget column {label}: content-message header {vals['content']} != {CW} (toml)")
        if "audio" in vals and AUDIO is not None and vals["audio"] != [AUDIO]:
            fail("AC 1", f"budget column {label}: audio {vals['audio']} != {AUDIO} (6 x 48 x 4)")
        if "cksum" in vals and CKSUM is not None and vals["cksum"] != [CKSUM]:
            fail("AC 1", f"budget column {label}: checksum {vals['cksum']} != {CKSUM} (toml)")
        ann_sum = sum(vals.get("ann", [0]))
        if ANN_MAX is not None and ann_sum not in ALLOWED_ANN_BYTES:
            fail("AC 2", f"budget column {label}: announcement rows sum to {ann_sum}, which is neither "
                         f"the count byte alone nor the count plus 12 bytes per announcement (up to {ANN_MAX})")
        ext_sum = sum(vals.get("ext", [0]))
        if ext_sum not in (0, EXT):
            fail("AC 2", f"budget column {label}: extension {ext_sum} != 0 or {EXT} (toml)")
        bound = PAY_FB if kind == "fb" else PAY_DEF
        if total is not None and total > bound:
            fail("AC 1", f"budget column {label}: total {total} exceeds the {bound}-byte bound")
        if spare != bound - total:
            fail("AC 1", f"budget column {label}: spare {spare} != {bound} - {total} = {bound - total}")
        if kind == "default":
            if ann_sum != ANN_COUNT or ext_sum != 0:
                fail("AC 1", f"default column {label} must hold the mandatory parts only "
                             "(announcement count byte, no extension)")
            if total != MANDATORY + AUDIO:
                fail("AC 1", f"default column {label}: total {total} != mandatory {MANDATORY} + audio {AUDIO}")
        if kind == "worst" and ann_sum != ANN_MAX:
            fail("AC 2", f"worst column {label}: announcements are not at maximum ({ann_sum} != {ANN_MAX})")
        if kind == "fb":
            non_audio = total - AUDIO
            if non_audio > CAP:
                fail("AC 3", f"fallback column {label}: headers plus checksum are {non_audio} bytes, "
                             f"above the {CAP}-byte cap of C-137")
        CASES.append({"kind": kind, "label": label, "total": total, "spare": spare, "bound": bound,
                      "ann": ann_sum, "ext": ext_sum, "table": t})

if not CASES:
    fail("AC 1", "no budget table: a 'Total' row with numeric columns in the Size budget section")

defaults = [c for c in CASES if c["kind"] == "default"]
worst_ext = [c for c in CASES if c["kind"] == "worst" and c["ext"] == EXT and c["ann"] == ANN_MAX]
worst_noext = [c for c in CASES if c["kind"] == "worst" and c["ext"] == 0 and c["ann"] == ANN_MAX]
fallbacks = [c for c in CASES if c["kind"] == "fb"]
if not defaults:
    fail("AC 1", f"no default 1 472 budget column (mandatory parts only: total {MANDATORY + AUDIO if MANDATORY is not None else '?'})")
if not worst_ext:
    fail("AC 2", f"no worst-case 1 472 column with all announcements and the extension (total {MAX_NON_AUDIO + AUDIO if MAX_NON_AUDIO is not None else '?'})")
if not worst_noext:
    fail("AC 2", "no worst-case 1 472 column with all announcements and no extension")
if not fallbacks:
    fail("AC 3", "no 1 280-byte fallback budget column")
for c in CASES:
    if c["kind"] != "fb" and c["bound"] != PAY_DEF:
        fail("AC 1", "a 1 472 column has the wrong bound")

for need_word, ac in (("C-100", "AC 1"), ("C-06", "AC 3"), ("C-137", "AC 3")):
    if not re.search(r"\b%s\b" % re.escape(need_word), sec_text):
        fail(ac, f"the Size budget section does not cite {need_word}")
if not re.search(r"1\s?472", sec_text) or not re.search(r"1\s?280", sec_text) or not re.search(r"1\s?252", sec_text):
    fail("AC 1", "the section does not state the 1 472, 1 280 and 1 252 byte figures")

# ---------------------------------------------------------------------------
# AC 3: omission rule for the fallback (text), and the sender-obligation rule.
# ---------------------------------------------------------------------------
omit_text = " ".join(l for l, f in zip(sec_lines, sec_fence) if not f and re.search(r"\bomit", l, re.I))
omit_text = norm(omit_text)
# Form (ii): an explicit statement that no optional part is omitted for size on a fallback path.
# The ticket's AC 3 is met by an empty list; the fallback case checks below still bound every column.
NOTHING_OMITTED = re.compile(
    r"\b(?:no|nothing|none|never)\b[^.;]{0,80}?\bomitted\b|\bnot omitted\b|\bomitted\b[^.;]{0,20}?\bnever\b",
    re.I)
nothing_omitted = any(re.search(r"\bfallback\b", sent, re.I) and NOTHING_OMITTED.search(sent)
                      for sent in re.split(r"(?<=[.;])\s+", sec_text))
if nothing_omitted:
    pass
elif not omit_text:
    fail("AC 3", "the section states neither an omission rule nor that no optional part is omitted for size "
                 "on a fallback path (no line says 'omit')")
else:
    if not re.search(r"announcement", omit_text, re.I):
        fail("AC 3", "the omission rule does not say which announcements are omitted")
    if not re.search(r"measurement|extension", omit_text, re.I):
        fail("AC 3", "the omission rule does not say what happens to the measurement extension")
    if not re.search(r"controller", omit_text, re.I):
        fail("AC 3", "the omission rule does not say what happens to controller messages")
    if not re.search(r"priorit|in this order|\bfirst\b|\bthen\b|\bbefore\b|\bordered\b", omit_text, re.I):
        fail("AC 3", "the omission rule states no priority (which part goes first)")
    for c in fallbacks:
        omits_ann = c["ann"] != ANN_MAX
        omits_ext = c["ext"] == 0
        if (omits_ann or omits_ext) and not re.search(r"\bamend|\bscop(?:e|es|ed|ing)\b|\bsupersed", sec_text, re.I):
            fail("AC 3", f"fallback column {c['label']} omits announcements or the extension, but the section "
                         "does not say that it amends or scopes the landed sender obligations (C-44, 004-10)")
if not fallbacks:
    fail("AC 3", "no 1 280-byte fallback budget column")

# ---------------------------------------------------------------------------
# AC 4: the verdict against the falsifier.
# ---------------------------------------------------------------------------
verdict_chunks = []
for i, line in enumerate(sec_lines):
    if sec_fence[i] or "verdict" not in line.lower():
        continue
    k = i
    while k > 0 and sec_lines[k - 1].strip() and not sec_fence[k - 1]:
        k -= 1
    j = i
    while j + 1 < len(sec_lines) and sec_lines[j + 1].strip() and not sec_fence[j + 1]:
        j += 1
    verdict_chunks.append(norm(" ".join(sec_lines[k:j + 1])))
vtext = " ".join(dict.fromkeys(verdict_chunks))
if not vtext:
    fail("AC 4", "no verdict: the section answers the open sub-question with no paragraph containing 'verdict'")
else:
    holds = bool(re.search(r"C-137 holds|\bholds\b|not falsified|\bconfirmed\b", vtext, re.I))
    falsified = bool(re.search(r"(?<!not )\bfalsified\b", vtext, re.I))
    if MANDATORY is not None and CAP is not None:
        if MANDATORY <= CAP:
            if falsified:
                fail("AC 4", f"the verdict says falsified, but the mandatory headers and checksum are "
                             f"{MANDATORY} bytes, within the {CAP}-byte cap")
            if not holds:
                fail("AC 4", "the verdict does not say C-137 holds (or 'not falsified' / 'confirmed')")
        else:
            if holds and not falsified:
                fail("AC 4", f"the verdict holds, but the mandatory headers and checksum are {MANDATORY} bytes, "
                             f"above the {CAP}-byte cap")
            if not falsified:
                fail("AC 4", "the verdict must report C-137 falsified")
            if not re.search(r"5 channels|five channels", sec_text, re.I):
                fail("AC 4", "a falsified verdict must say C-04 falls to 5 channels at the fallback")
            if const_value("C-04") is None or first_int(const_value("C-04")) != 5:
                fail("AC 4", "a falsified verdict needs the constants table to show C-04 at 5 channels first")

# ---------------------------------------------------------------------------
# AC 5: the machine-readable totals.
# ---------------------------------------------------------------------------
budget_tables = [(n, v) for n, v in OTHERS.items() if "budget" in n.lower()]
BUDGET_FLAT = {}
for _name, _value in budget_tables:
    _items = _value if isinstance(_value, list) else [_value]
    for _block in _items:
        for _k, _v in _block.items():
            if isinstance(_v, int) and not isinstance(_v, bool):
                BUDGET_FLAT[_k] = _v

if not budget_tables:
    if overridden:
        print("NOTE AC 5: document paths overridden and wire.toml has no budget table; "
              "cargo run --bin wire-check skipped")
    else:
        try:
            proc = subprocess.run(["cargo", "run", "--quiet", "--bin", "wire-check"],
                                  capture_output=True, text=True, timeout=600)
            out = proc.stdout + proc.stderr
            if proc.returncode != 0:
                fail("AC 5", "cargo run --bin wire-check did not pass (no budget table in wire.toml, so "
                             "the 004-02 checker must derive and print the totals):\n" + out.strip()[-400:])
            else:
                for c in CASES:
                    if c["kind"] in ("default", "fb") and str(c["total"]) not in out:
                        fail("AC 5", f"wire-check does not print the {c['kind']} total {c['total']}")
        except (OSError, subprocess.TimeoutExpired) as error:
            fail("AC 5", f"could not run cargo run --bin wire-check: {error}")

# ---------------------------------------------------------------------------
# AC 6: the controller allowance inside the cap, and the sparse arithmetic.
# ---------------------------------------------------------------------------
allow_tables = [t for t in TABLES_SEC if any(re.search(r"allow", h, re.I) for h in t["header"])]
allowances = []
allow_rows = []
if not allow_tables:
    fail("AC 6", "no controller allowance table (a header cell containing 'allow')")
else:
    t = allow_tables[0]
    hdr = t["header"]

    def col(pattern, exclude=None):
        for i, h in enumerate(hdr):
            if re.search(pattern, h, re.I) and not (exclude and re.search(exclude, h, re.I)):
                return i
        return None

    bi = col(r"bound|cap|payload|limit")
    ai = col(r"audio")
    ci = col(r"count|header", exclude=r"bound|cap|payload")
    ki = col(r"checksum|crc")
    li = col(r"allow")
    if None in (bi, ai, ci, ki, li):
        fail("AC 6", "the allowance table needs columns for the bound (or cap or payload), audio, counted "
                     "headers, checksum and allowance")
    else:
        paths_seen = set()
        for r in t["rows"]:
            if not r or label_of(r[0]) == "":
                continue
            lab = r[0]
            if re.search(r"1\s?472", lab):
                path = "default"
            elif re.search(r"1\s?280|1\s?252|fallback", lab, re.I):
                path = "fb"
            else:
                fail("AC 6", f"allowance row {lab!r} names neither the 1 472 path nor the fallback")
                continue
            vals = [parse_num(r[i]) if i < len(r) else None for i in (bi, ai, ci, ki, li)]
            if None in vals:
                fail("AC 6", f"allowance row {lab!r} has a missing number")
                continue
            bound, audio, counted, cksum, allow = vals
            paths_seen.add(path)
            # The bound is a path payload (1 472 or 1 252) with the audio subtracted, or the
            # 100-byte header cap itself, which holds the headers and checksum only.
            if bound == CAP:
                expect = CAP - counted - cksum
                shape = "the 100-byte cap less the counted headers and checksum"
            elif bound in (PAY_DEF, FB_PAY):
                expect = bound - audio - counted - cksum
                shape = f"{bound} less audio, counted headers and checksum"
            else:
                fail("AC 6", f"allowance row {lab!r}: bound {bound} is neither the cap {CAP} nor a path payload "
                             f"({PAY_DEF} or {FB_PAY})")
                continue
            if path == "default" and bound not in (CAP, PAY_DEF):
                fail("AC 6", f"allowance row {lab!r}: the 1 472 path takes the cap or {PAY_DEF}")
            if path == "fb" and bound not in (CAP, FB_PAY):
                fail("AC 6", f"allowance row {lab!r}: the fallback takes the cap or {FB_PAY}")
            if audio != AUDIO:
                fail("AC 6", f"allowance row {lab!r}: audio {audio} != {AUDIO}")
            if cksum != CKSUM:
                fail("AC 6", f"allowance row {lab!r}: checksum {cksum} != {CKSUM} (toml)")
            if counted not in ALLOWED_COUNTED:
                fail("AC 6", f"allowance row {lab!r}: counted headers {counted} are not the mandatory parts "
                             "plus an announcement count and the extension (0 or present)")
            if allow != expect:
                fail("AC 6", f"allowance row {lab!r}: {allow} != {shape} = {expect}")
            if allow < 0:
                fail("AC 6", f"allowance row {lab!r} is negative")
            allowances.append(allow)
            allow_rows.append((lab.lower(), allow))
        for p in ("default", "fb"):
            if p not in paths_seen:
                fail("AC 6", "the allowance is not stated at both path sizes (1 472 and fallback)")

if not re.search(r"upper bound|most a source may share", sec_text, re.I):
    fail("AC 6", "the section does not say the allowance is an upper bound (the most a source may share)")
if not re.search(r"second stream", sec_text, re.I) or "C-181" not in sec_text:
    fail("AC 6", "the section does not say a source may always use a second stream (C-181)")
if "C-205" not in sec_text or not any(
        re.search(r"audio member", s, re.I) and re.search(r"\bno controller|counts? no controller|carries? no controller", s, re.I)
        for s in re.split(r"(?<=[.;])\s+", sec_text)):
    fail("AC 6", "the section does not say a bundle source's audio member counts no controller messages (C-205)")

sparse_ok = False
cands = {(a - TRANSPORT - SP_HDR) // SP_ENTRY for a in allowances if a >= TRANSPORT + SP_HDR}
for s in re.split(r"(?<=[.;])\s+", sec_text):
    if not (re.search(r"transport", s, re.I) and re.search(r"sparse", s, re.I)):
        continue
    nums = {to_int(m.group(0)) for m in NUM_RE.finditer(s)}
    nums |= {WORD_NUM[w] for w in re.findall(r"[a-z]+", s.lower()) if w in WORD_NUM}
    if TRANSPORT in nums and SP_ENTRY in nums and nums & cands:
        sparse_ok = True
        break
if not sparse_ok:
    fail("AC 6", f"no sentence gives the sparse entries that fit beside a transport state ({TRANSPORT} bytes) "
                 f"inside an allowance, with the sparse entry size {SP_ENTRY} bytes and header {SP_HDR} bytes")

# ---------------------------------------------------------------------------
# AC 5 (keys): every toml budget key is bound to its case by name tokens. Key spellings are
# not pinned: a key matches a case when its tokens satisfy the case's groups (any alternative
# of a group) and avoid its forbidden tokens. Its value must equal the computed figure for
# that case. Unknown integer keys are allowed; a key whose name contains "allowance" must
# equal a computed allowance from the controller allowance table (AC 6).
# ---------------------------------------------------------------------------
def key_tokens(name):
    return {t for t in re.split(r"[^a-z0-9]+", name.lower()) if t}


def tok_match(tokens, groups, forb):
    return all(tokens & g for g in groups) and not (tokens & forb)


ROLE = {"total": {"total"}, "spare": {"spare", "margin"}}
ANN_WORDS = {0: {"mandatory", "bare", "plain", "none"},
             1: {"one", "1"},
             3: {"three", "3"},
             4: {"four", "4", "all", "maximum", "full"}}


def case_groups(c):
    if c["kind"] == "default":
        return [{"default", "mandatory", "nominal", "1472"}], {"fallback", "worst"}
    if c["kind"] == "worst":
        if c["ext"]:
            return [{"worst", "maximum", "max"}], {"fallback", "default", "no", "without"}
        return [{"worst", "maximum", "max"}, {"no", "without", "absent"}], {"fallback", "default"}
    k = (c["ann"] - ANN_COUNT) // ENTRY if ENTRY else -1
    groups = [{"fallback"}, ANN_WORDS.get(k, {str(k)})]
    forb = {"default", "worst"}
    if c["ext"]:
        groups.append({"extension", "measurement"})
    else:
        forb = forb | {"extension", "measurement"}
    return groups, forb


PART_SPECS = [
    ("packet header", [{"packet"}, {"header"}], {"content", "total", "spare", "allowance", "fallback"}, PKT),
    ("slice header", [{"slice"}, {"header"}], {"total", "spare", "allowance"}, SLICE),
    ("announcement count", [{"announcement"}, {"count"}], {"total", "spare", "allowance", "container", "max", "maximum"}, ANN_COUNT),
    ("announcement container maximum", [{"announcement"}, {"container", "max", "maximum"}], {"count", "total", "spare", "allowance"}, ANN_MAX),
    ("measurement extension", [{"measurement", "extension"}], {"allowance", "total", "spare"}, EXT),
    ("content-message header", [{"content"}, {"header"}], {"packet", "slice", "total", "spare", "allowance"}, CW),
    ("checksum", [{"checksum", "crc"}], {"total", "spare", "allowance"}, CKSUM),
    ("audio", [{"audio"}], {"total", "spare", "allowance", "mandatory", "maximum"}, AUDIO),
    ("path payload", [{"payload"}], {"fallback", "allowance", "total", "spare"}, PAY_DEF),
    ("fallback path payload", [{"payload"}, {"fallback"}], {"allowance", "total", "spare"}, PAY_FB),
    ("mandatory beside audio", [{"mandatory"}], {"total", "spare", "allowance", "fallback", "default", "worst"}, MANDATORY),
    ("maximum beside audio", [{"maximum"}], {"container", "announcement", "total", "spare", "allowance", "worst", "fallback", "default"}, MAX_NON_AUDIO),
]

if budget_tables:
    key_fails = 0
    for c in CASES:
        groups, forb = case_groups(c)
        for role, expected in (("total", c["total"]), ("spare", c["spare"])):
            cands = [(k, v) for k, v in BUDGET_FLAT.items()
                     if tok_match(key_tokens(k), groups + [ROLE[role]], forb)]
            if not cands:
                fail("AC 5", f"no wire.toml budget key for the {c['kind']} case (announcements {c['ann']}, "
                             f"extension {c['ext']}) and its {role}")
                key_fails += 1
                continue
            for k, v in cands:
                if v != expected:
                    fail("AC 5", f"wire.toml key {k!r} = {v}, but the {c['kind']} case (announcements {c['ann']}, "
                                 f"extension {c['ext']}) has {role} {expected}")
                    key_fails += 1
    for label, groups, forb, expected in PART_SPECS:
        if expected is None:
            continue
        for k, v in BUDGET_FLAT.items():
            if tok_match(key_tokens(k), groups, forb) and v != expected:
                fail("AC 5", f"wire.toml key {k!r} = {v}, but the {label} is {expected} (landed table)")
    for k, v in BUDGET_FLAT.items():
        toks = key_tokens(k)
        if "allowance" not in toks:
            continue
        if toks & {"not", "without", "non", "no"}:
            want = {a for lab, a in allow_rows if "could not" in lab}
            role = "could not carry the extension"
        elif toks & {"could", "carry", "with", "measuring", "measure"}:
            want = {a for lab, a in allow_rows if "could carry" in lab and "could not" not in lab}
            role = "could carry the extension"
        else:
            want = set(allowances)
            role = "any computed allowance"
        if not want:
            fail("AC 5", f"wire.toml key {k!r} names an allowance for a stream that {role}, "
                         "but wire.md has no such row")
        elif v not in want:
            fail("AC 5", f"wire.toml key {k!r} = {v}, but the row for {role} is {sorted(want)}")
    if not allowances and any("allowance" in key_tokens(k) for k in BUDGET_FLAT):
        fail("AC 5", "wire.toml names an allowance but wire.md has no computed allowance table")

# ---------------------------------------------------------------------------
# DoD: evidence tags and open items in the section.
# ---------------------------------------------------------------------------
EVIDENCE = re.compile(r"\[(?:code|measured|kernel-doc|standard|memory|decision)\]", re.I)
units = []
cur = []
for i, line in enumerate(sec_lines):
    s = line.strip()
    if sec_fence[i]:
        continue
    if not s or s.startswith("#") or is_table_line(line):
        if cur:
            units.append(" ".join(cur))
            cur = []
        continue
    if re.match(r"^([-*+]\s|\d+\.\s)", s) and cur:
        units.append(" ".join(cur))
        cur = []
    cur.append(s)
if cur:
    units.append(" ".join(cur))
for u in units:
    words = re.findall(r"[A-Za-z0-9]+", u)
    if len(words) >= 5 and not EVIDENCE.search(u):
        fail("DoD", "a prose paragraph of 5+ words has no evidence tag: " + u[:80])
    stripped = re.sub(r"^[\s*_]+", "", u)
    if re.match(r"open item", stripped, re.I):
        if not re.search(r"owner", u, re.I) or not re.search(r"falsifier", u, re.I):
            fail("DoD", "an open item in the section names no owner or no falsifier: " + u[:80])

# ---------------------------------------------------------------------------
# Pointers and landed sentences outside the section.
# ---------------------------------------------------------------------------
wire_norm = norm(wire)
glob_norm = norm(" ".join(glob_lines))

if "open item owned by 004-14" in wire_norm:
    fail("pointer", "the string 'open item owned by 004-14' remains in wire.md; the Size budget section must close it")
if re.search(r"But see[^)]{0,200}?\bBudget\b", wire_norm):
    fail("pointer", "a 'But see ... Budget' pointer remains in wire.md; resolve it or cite the Size budget section")
if re.search(r"(?:open item|overflow)[^.]{0,120}?004-14|004-14[^.]{0,40}?\(Budget\)", glob_norm, re.I):
    fail("pointer", "an unresolved pointer to 004-14 (Budget) remains outside the Size budget section")

for line in glob_lines:
    if "provisional" in line.lower() and re.search(r"004-14|shared-packet|\bcap\b", line, re.I):
        if not (re.search(r"stays provisional", sec_text, re.I) and re.search(r"\bbecause\b", sec_text, re.I)):
            fail("pointer", "a 'provisional' cap or shared-packet line remains: " + norm(line)[:90])
            break

LANDED = [
    "All four announcements fit inside the 100-byte cap",
    "the container never blocks that cadence",
    "the extension fits inside the cap in every v1 packet",
    "The header cap holds with no fallback rule",
    "no v1 sender omits it on a fallback path",
]
for phrase in LANDED:
    for line in glob_lines:
        if phrase.lower() in norm(line).lower() and not re.search(r"size budget", line, re.I):
            fail("pointer", f"the landed sentence {phrase!r} is neither removed nor cited to the Size budget section")
            break

for line in glob_lines:
    stripped = re.sub(r"^[\s*_]+", "", line.strip())
    if re.match(r"Open item \(header cap\)", stripped, re.I):
        if not re.search(r"\bclosed\b|\bresolved\b|\banswered\b", line, re.I):
            fail("pointer", "the header-cap open item of Content messages, Budget is not marked closed or resolved")

if fails:
    for ac, msg_text in fails:
        print(f"FAIL {ac}: {msg_text}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print("OK: 6/6 acceptance criteria for 004-14, the budget totals, the verdict and the pointers, and the DoD tags")
PY
PY_RC=$?

if [ "$PY_RC" -ne 0 ]; then
    exit 1
fi
exit 0
