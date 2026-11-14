#!/usr/bin/env bash
# 004-01 — wire.md skeleton and encoding conventions.
#
# RED-phase document check. It validates doc/design/spec/wire.md against every
# acceptance criterion of the ticket. The constants in doc/design/02-constants.md
# are the single source of truth: every value listed in wire.md is compared
# against that table, and the table wins.
#
# WIRE_MD / CONSTANTS_MD may override the paths (used to self-test the checker
# on fixtures); the defaults are the real design files.
set -u
cd "$(dirname "$0")/.."

WIRE="${WIRE_MD:-doc/design/spec/wire.md}"
CONST="${CONSTANTS_MD:-doc/design/02-constants.md}"

python3 - "$WIRE" "$CONST" <<'PY'
import re
import sys

wire_path, const_path = sys.argv[1], sys.argv[2]

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


def norm_cell(s):
    # Ignore parenthetical units, e.g. "Offset (bytes)" -> "offset".
    s = re.sub(r"\([^)]*\)", "", s)
    return re.sub(r"[^a-z0-9]", "", s.lower())


wire = read_text(wire_path)
consts_text = read_text(const_path)

if wire is None:
    print(f"FAIL AC1: {wire_path} does not exist")
    for ac in range(2, 8):
        print(f"FAIL AC{ac}: cannot check — {wire_path} does not exist")
    sys.exit(1)

# ---------------------------------------------------------------------------
# Headings and sections (fenced code excluded)
# ---------------------------------------------------------------------------
headings = []
in_fence = False
for idx, line in enumerate(wire.splitlines()):
    if re.match(r"^\s*```", line):
        in_fence = not in_fence
        continue
    if in_fence:
        continue
    m = re.match(r"^(#{1,6})\s+(.*?)\s*#*\s*$", line)
    if m:
        headings.append((idx, len(m.group(1)), m.group(2).strip()))


def section_of(i):
    level = headings[i][1]
    start = headings[i][0] + 1
    end = len(wire.splitlines())
    for j in range(i + 1, len(headings)):
        if headings[j][1] <= level:
            end = headings[j][0]
            break
    return "\n".join(wire.splitlines()[start:end])


def section_for_title(pred):
    for i, (_, _, title) in enumerate(headings):
        if pred(title):
            return i, section_of(i)
    return None, None


def full_section(i):
    return headings[i][2] + "\n" + section_of(i)


# ---------------------------------------------------------------------------
# AC 1 — one byte order for all multi-byte integers, [decision], justification
# ---------------------------------------------------------------------------
ac1 = True
i_bo, _ = section_for_title(lambda t: re.search(r"byte[\s-]*order", t, re.I))
if i_bo is None:
    fail(1, "no byte-order section heading (expected e.g. 'Byte order and alignment')")
    ac1 = False
else:
    bo_text = full_section(i_bo)
    order_pats = [
        r"\b(?:all|every)\s+(?:multi[-\s]?byte|multibyte)\b[^.\n]{0,160}?\b(?P<ord>little|big)[-\s]endian\b",
        r"\b(?P<ord>little|big)[-\s]endian\b[^.\n]{0,160}?\b(?:for|on|across|of)\s+(?:all|every)\s+(?:multi[-\s]?byte|multibyte)\b",
        r"\bbyte[\s-]*order\b[^.\n]{0,80}?(?::|is|are|=|—|–|-)\s*(?P<ord>little|big)[-\s]endian\b",
        r"\|\s*(?:byte[\s-]*order|endianness)\s*\|\s*(?P<ord>little|big)[-\s]endian\b",
        r"\b(?:multi[-\s]?byte|multibyte)\b[^.\n]{0,100}?\b(?:integers?|values?|fields?|quantities)\b[^.\n]{0,100}?\b(?P<ord>little|big)[-\s]endian\b",
    ]
    orders = set()
    for pat in order_pats:
        for m in re.finditer(pat, bo_text, re.I):
            prefix = bo_text[max(0, m.start() - 24) : m.start()]
            if re.search(
                r"\b(?:not|never|rather\s+than|instead\s+of|unlike|rejected)\s*$",
                prefix,
                re.I,
            ):
                continue
            orders.add(m.group("ord").lower())

    # Accepted aliases that name an order without the words little/big-endian.
    aliases = [
        (r"\bnetwork\s+byte\s+order\b|\bmost[-\s]+significant[-\s]+byte[-\s]+first\b|\bMSB[-\s]+first\b", "big"),
        (r"\bleast[-\s]+significant[-\s]+byte[-\s]+first\b|\bLSB[-\s]+first\b", "little"),
    ]
    scoped_any = re.search(
        r"\b(?:all|every)\s+(?:multi[-\s]?byte|multibyte)"
        r"|\b(?:multi[-\s]?byte|multibyte)\s+(?:integers?|fields?|values?|quantities?)",
        bo_text,
        re.I,
    )
    if scoped_any:
        for pat, order in aliases:
            if re.search(pat, bo_text, re.I):
                orders.add(order)

    if not orders:
        # Fallback for phrasings such as "the wire uses big-endian byte order
        # for every multi-byte integer" that scope the order without the
        # declaration word order the patterns above expect.
        mentioned = set(
            o.lower() for o in re.findall(r"\b(little|big)[-\s]endian\b", bo_text, re.I)
        )
        scoped = re.search(
            r"\b(?:all|every)\s+(?:multi[-\s]?byte|multibyte)"
            r"|\b(?:multi[-\s]?byte|multibyte)\s+(?:integers?|fields?|values?|quantities?)",
            bo_text,
            re.I,
        )
        if len(mentioned) == 1 and scoped:
            orders = mentioned
    if not orders:
        fail(1, "byte-order section does not state which single order applies to all multi-byte wire integers")
        ac1 = False
    elif len(orders) > 1:
        fail(1, f"byte-order section declares more than one order: {sorted(orders)}")
        ac1 = False
    if "[decision]" not in bo_text:
        fail(1, "byte-order section does not carry the [decision] tag")
        ac1 = False
    if not re.search(
        r"\b(justif\w*|rationale|because|since|reason\w*|we choose|chosen|decided|"
        r"therefore|thus|so\s+that|ensures?|avoids?|simplif\w*|minimi[sz]\w*|"
        r"makes?|allows?|matches?|consistent|portable|network\s+byte\s+order|"
        r"most[-\s]+significant|byte\s+first)\b",
        bo_text,
        re.I,
    ):
        fail(1, "byte-order section has no justification")
        ac1 = False

if ac1:
    print("PASS AC1: byte-order section declares one order, tagged [decision], justified")

# ---------------------------------------------------------------------------
# AC 2 — alignment stated for widths 1, 2, 4, 8, 16 bytes and bit fields
# ---------------------------------------------------------------------------
ac2 = True
i_al, _ = section_for_title(lambda t: re.search(r"align", t, re.I))
if i_al is None:
    fail(2, "no alignment section heading")
    ac2 = False
else:
    al_text = full_section(i_al)
    satisfied = set()

    def answer_present(cell):
        c = cell.strip().strip("*_`")
        return bool(c) and c.lower() not in {"—", "–", "-", "n/a", "na", "none?", "?"}

    def mark_widths(text):
        for w in re.findall(r"(?<![\d.])(?:1|2|4|8|16)(?![\d.])", text):
            satisfied.add(w)
        if re.search(r"sub[-\s]?byte|bit[-\s]?field|bitfield|\bbits?\b", text, re.I):
            satisfied.add("bit")

    # Prefer an explicit width/alignment table.
    for _, header, rows in parse_tables(al_text):
        wcol = acol = None
        for k, h in enumerate(header):
            hl = h.lower()
            if wcol is None and re.search(r"width|bytes?|size", hl):
                wcol = k
            if acol is None and re.search(r"align|guarantee|padding|padded|packed|natural", hl):
                acol = k
        if wcol is None or acol is None:
            continue
        for row in rows:
            if len(row) <= max(wcol, acol):
                continue
            if answer_present(row[acol]):
                mark_widths(row[wcol])
                if re.search(r"sub[-\s]?byte|bit[-\s]?field|bitfield", row[wcol], re.I):
                    satisfied.add("bit")

    # Prose lines also count if they connect a width to an alignment statement.
    for line in al_text.splitlines():
        if line.lstrip().startswith("|"):
            continue
        if not re.search(r"align|guarantee|padding|padded|packed", line, re.I):
            continue
        mark_widths(line)

    for w, label in [
        ("1", "1-byte fields"),
        ("2", "2-byte fields"),
        ("4", "4-byte fields"),
        ("8", "8-byte fields"),
        ("16", "16-byte fields"),
        ("bit", "sub-byte bit fields"),
    ]:
        if w not in satisfied:
            fail(2, f"alignment section does not state whether natural alignment is guaranteed for {label}")
            ac2 = False

if ac2:
    print("PASS AC2: alignment stated for widths 1, 2, 4, 8, 16 and sub-byte bit fields")

# ---------------------------------------------------------------------------
# AC 3 — f32 and Q1.31 encoded bit-exactly, each under a bit-level-encoding
#        heading and 4 bytes wide
# ---------------------------------------------------------------------------
ac3 = True
bit_level = [
    i
    for i, (_, _, title) in enumerate(headings)
    if re.search(r"bit[-\s]?level\s+encoding", title, re.I)
]
if not bit_level:
    fail(3, "no 'bit-level encoding' heading for f32 / Q1.31")
    ac3 = False
else:

    def inside_bit_level(i):
        for j in range(i - 1, -1, -1):
            if headings[j][1] < headings[i][1]:
                return bool(
                    re.search(r"bit[-\s]?level\s+encoding", headings[j][2], re.I)
                )
        return False

    fmt_pats = [("f32", r"\bf32\b"), ("Q1.31", r"\bq1[.\s_]?31\b")]
    for label, pat in fmt_pats:
        named = False
        for i, (_, _, title) in enumerate(headings):
            if re.search(pat, title, re.I) and (
                re.search(r"bit[-\s]?level\s+encoding", title, re.I) or inside_bit_level(i)
            ):
                named = True
                break
        if not named:
            fail(3, f"no bit-level-encoding heading (or subsection) names {label}")
            ac3 = False

    bl_text = "\n\n".join(full_section(i) for i in bit_level)

    def nearby(text, pat_a, pat_b, window=320):
        for m in re.finditer(pat_a, text, re.I):
            lo = max(0, m.start() - window)
            hi = min(len(text), m.end() + window)
            if re.search(pat_b, text[lo:hi], re.I):
                return True
        return False

    if not re.search(r"\bf32\b", bl_text, re.I):
        fail(3, "bit-level encoding section does not define f32")
        ac3 = False
    if not re.search(r"\bq1[.\s_]?31\b", bl_text, re.I):
        fail(3, "bit-level encoding section does not define Q1.31")
        ac3 = False

    four = r"\b4\s*[- ]?bytes?\b|\b32\s*[- ]?bits?\b"
    if not nearby(bl_text, r"\bf32\b", four, 400):
        fail(3, "f32 is not stated as 4 bytes (32 bits)")
        ac3 = False
    if not nearby(bl_text, r"\bq1[.\s_]?31\b", four, 400):
        fail(3, "Q1.31 is not stated as 4 bytes (32 bits)")
        ac3 = False

    if not nearby(bl_text, r"\bf32\b", r"ieee\s*[- ]?754|binary32", 500):
        fail(3, "f32 encoding does not cite IEEE 754 binary32")
        ac3 = False
    if not nearby(bl_text, r"\bf32\b", r"\bexponent\b", 500):
        fail(3, "f32 bit layout does not name the exponent field")
        ac3 = False
    if not nearby(bl_text, r"\bf32\b", r"\bmantissa\b|\bfraction\b", 500):
        fail(3, "f32 bit layout does not name the mantissa/fraction field")
        ac3 = False
    if not nearby(bl_text, r"\bf32\b", r"\bsign\b", 500):
        fail(3, "f32 bit layout does not name the sign bit")
        ac3 = False

    if not nearby(bl_text, r"\bq1[.\s_]?31\b", r"two'?s\s+complement|2'?s\s+complement", 500):
        fail(3, "Q1.31 encoding does not state two's complement")
        ac3 = False
    if not nearby(bl_text, r"\bq1[.\s_]?31\b", r"\b31\b", 400):
        fail(3, "Q1.31 encoding does not state its 31 fractional bits")
        ac3 = False
    if not nearby(bl_text, r"\bq1[.\s_]?31\b", r"fraction\w*|\bscale\b|2\s*\^?\s*[-−]?\s*31", 500):
        fail(3, "Q1.31 encoding does not state its fractional scale")
        ac3 = False

if ac3:
    print("PASS AC3: f32 and Q1.31 defined bit-exactly under bit-level-encoding sections")

# ---------------------------------------------------------------------------
# AC 4 — field-table format: exactly offset, width, type, meaning, valid range;
#        variable-length parts have a stated maximum count
# ---------------------------------------------------------------------------
ac4 = True
field_sections = [
    (i, section_of(i))
    for i, (_, _, title) in enumerate(headings)
    if re.search(r"field[\s-]*table", title, re.I)
]
if not field_sections:
    fail(4, "no field-table format section heading")
    ac4 = False
else:
    expected = ["offset", "width", "type", "meaning", "validrange"]
    found = False
    for _, sec in field_sections:
        for _, header, _ in parse_tables(sec):
            got = [norm_cell(h) for h in header]
            if got == expected:
                found = True
                break
        if found:
            break
    if not found:
        fail(
            4,
            "no table with exactly the five columns offset | width | type | meaning | valid range",
        )
        ac4 = False

    combined = "\n\n".join(sec for _, sec in field_sections)
    var_rule = False
    for m in re.finditer(r"variable[\s-]*length", combined, re.I):
        lo = max(0, m.start() - 400)
        hi = min(len(combined), m.end() + 400)
        window = combined[lo:hi]
        if re.search(r"max(?:imum)?", window, re.I) and re.search(
            r"count|fixed|bounded|limit", window, re.I
        ):
            var_rule = True
            break
    if not var_rule:
        fail(
            4,
            "no rule that every variable-length part has a stated maximum count",
        )
        ac4 = False

if ac4:
    print("PASS AC4: five-column field table and bounded variable-length rule present")

# ---------------------------------------------------------------------------
# AC 5 — the 22 placeholder sections are present as (distinct) headings
# ---------------------------------------------------------------------------
placeholders = [
    ("rejection reasons", r"rejection[-\s]+reasons?"),
    ("versioning", r"version(?:ing)?\b"),
    ("reserved fields", r"reserved[-\s]+fields?"),
    ("media packet header", r"media[-\s]+packet[-\s]+header"),
    ("slice header", r"slice[-\s]+header"),
    ("announcements", r"announcements?\b"),
    ("measurement extension", r"measurement[-\s]+extension"),
    ("content messages", r"content[-\s]+messages?"),
    ("checksum", r"checksum"),
    ("size budget", r"size[-\s]+budget"),
    ("hello", r"\bhello\b"),
    ("election messages", r"election\b"),
    ("admission messages", r"admission\b"),
    ("manifest", r"\bmanifest\b"),
    ("path-MTU probe pair", r"path[-\s]*mtu|probe\s+pair"),
    ("regime stamp", r"regime[-\s]+stamp"),
    ("commissioning window messages", r"commissioning[-\s]+window"),
    ("sync request and reply", r"sync\s+requests?[^|]{0,20}repl(?:y|ies)|sync\s+request\s*/\s*reply"),
    ("receiver report", r"receiver[-\s]+reports?"),
    ("one-way probe", r"one[-\s]+way[-\s]+probe"),
    ("validity rules", r"validity[-\s]+rules?"),
    ("traceability table", r"traceability"),
]

ac5 = True
used = {}
for name, pat in placeholders:
    matches = [i for i, (_, _, title) in enumerate(headings) if re.search(pat, title, re.I)]
    if not matches:
        fail(5, f"placeholder section missing: {name}")
        ac5 = False
        continue
    i = matches[0]
    if i in used:
        fail(
            5,
            f"placeholder section {name!r} shares its heading with {used[i]!r}: "
            f"{headings[i][2]!r}",
        )
        ac5 = False
        continue
    used[i] = name
    body = section_of(i)
    if not re.search(r"to\s+be\s+filled", body, re.I):
        warn(5, f"placeholder {name!r} has no 'to be filled by 004-TT' note")
    elif not re.search(r"004-\d+", body):
        warn(5, f"placeholder {name!r} note does not name a 004-TT ticket")

if ac5:
    print(f"PASS AC5: all {len(placeholders)} placeholder sections present as headings")

# ---------------------------------------------------------------------------
# AC 6 — four-layer table with DSCP 40 / 48 / 46 / 40+46 and IPv4 TTL 1
# ---------------------------------------------------------------------------
ac6 = True
layer_table = None
for start, header, rows in parse_tables(wire):
    hl = [h.lower() for h in header]
    lcol = dcol = None
    for k, h in enumerate(hl):
        if lcol is None and "layer" in h:
            lcol = k
        if dcol is None and "dscp" in h:
            dcol = k
    if lcol is not None and dcol is not None:
        layer_table = (start, header, rows, lcol, dcol)
        break

if layer_table is None:
    fail(6, "no layer table with Layer and DSCP columns")
    ac6 = False
else:
    start, header, rows, lcol, dcol = layer_table
    cats = {
        "control": re.compile(r"control|discovery", re.I),
        "sync": re.compile(r"sync|clock", re.I),
        "media": re.compile(r"media", re.I),
        "measurement": re.compile(r"measurement", re.I),
    }
    seen = {}
    for row in rows:
        if len(row) <= max(lcol, dcol):
            continue
        layer_cell = row[lcol]
        for cat, pat in cats.items():
            if pat.search(layer_cell):
                seen.setdefault(cat, []).append(row[dcol])
                break
    if len(rows) != 4:
        fail(6, f"layer table has {len(rows)} rows, expected four layers")
        ac6 = False
    for cat in cats:
        if cat not in seen:
            fail(6, f"layer table has no {cat} row")
            ac6 = False
            continue
        if len(seen[cat]) != 1:
            fail(6, f"layer table has {len(seen[cat])} {cat} rows, expected one")
            ac6 = False
            continue
        ints = {int(x) for x in re.findall(r"\d+", seen[cat][0])}
        if cat == "control":
            ok = 40 in ints and 46 not in ints and 48 not in ints
            want = "CS5 (40)"
        elif cat == "sync":
            ok = 48 in ints and 40 not in ints and 46 not in ints
            want = "CS6 (48)"
        elif cat == "media":
            ok = 46 in ints and 40 not in ints and 48 not in ints
            want = "EF (46)"
        else:
            ok = 40 in ints and 46 in ints and 48 not in ints
            want = "reports CS5 (40), probes EF (46)"
        if not ok:
            fail(6, f"{cat} layer DSCP is {seen[cat][0]!r}, expected {want}")
            ac6 = False

    # IPv4 and multicast TTL 1 must be stated with the layer table.
    before = [h for h in headings if h[0] < start]
    if before:
        h = before[-1]
        table_section = "\n".join(wire.splitlines()[h[0] : start + 10])
        # include the rest of the section, not just ten lines
        idx = headings.index(h)
        table_section = "\n".join(wire.splitlines()[h[0] : headings[idx + 1][0]]) if idx + 1 < len(headings) else "\n".join(wire.splitlines()[h[0] :])
    else:
        table_section = "\n".join(wire.splitlines()[: start + 10])
    if not re.search(r"\bIPv4\b", table_section, re.I):
        fail(6, "layer overview does not state IPv4")
        ac6 = False
    if not re.search(r"\bmulticast\b", table_section, re.I):
        fail(6, "layer overview does not state multicast")
        ac6 = False
    if not re.search(r"\bTTL\s*(?:=|of\s*)?1\b", table_section, re.I):
        fail(6, "layer overview does not state multicast TTL 1")
        ac6 = False

if ac6:
    print("PASS AC6: four-layer table with DSCP 40/48/46/40+46 and IPv4 TTL 1")

# ---------------------------------------------------------------------------
# AC 7 — Constants subsection lists every required id, values match 02-constants
# ---------------------------------------------------------------------------
required = [f"C-{n:02d}" for n in range(1, 14)]
required += [f"C-{n:02d}" for n in range(17, 23)]
required += ["C-34", "C-40", "C-43", "C-44", "C-100", "C-137"]

ac7 = True
const_values = {}
if consts_text is None:
    fail(7, f"{const_path} does not exist; cannot compare constants")
    ac7 = False
else:
    for line in consts_text.splitlines():
        s = line.strip()
        if not s.startswith("|"):
            continue
        cells = split_row(s)
        if len(cells) >= 3 and re.fullmatch(r"C-\d+", cells[0]):
            const_values[cells[0]] = cells[2]

    i_const, const_sec = section_for_title(
        lambda t: re.fullmatch(r"(?:\d+[.)]?\s*)?(?:design\s+)?constants?(?:\s+subsection)?", t.strip(), re.I)
    )
    if i_const is None:
        i_const, const_sec = section_for_title(
            lambda t: re.search(r"\bconstants?\b", t, re.I)
        )
    if i_const is None:
        fail(7, "no Constants subsection")
        ac7 = False
    else:
        # Parse tables first, then fall back to per-line entries.
        entries = {}
        for _, header, rows in parse_tables(const_sec):
            idcol = valcol = None
            for k, h in enumerate(header):
                hn = norm_cell(h)
                if idcol is None and hn == "id":
                    idcol = k
                if valcol is None and "value" in hn:
                    valcol = k
            if idcol is None:
                continue
            for row in rows:
                if len(row) <= idcol:
                    continue
                cid = row[idcol].strip().strip("*`")
                if cid in required and cid not in entries:
                    if valcol is not None and len(row) > valcol:
                        entries[cid] = row[valcol]
                    else:
                        entries[cid] = " | ".join(row)
        for line in const_sec.splitlines():
            for cid in required:
                if cid in entries:
                    continue
                if re.search(rf"\b{re.escape(cid)}\b", line):
                    entries[cid] = line.strip()

        def numbers(text):
            text = text or ""
            text = re.sub(r"\bC-\d+\b", " ", text)
            text = text.replace("\u00a0", " ").replace("\u202f", " ")
            # 1,252 -> 1252 (thousands separator), before tokenising
            text = re.sub(r"(?<=\d),(?=\d{3}(?!\d))", "", text)
            out = []
            for m in re.finditer(
                r"(?<![\w.])\d+(?:[ ]\d{3})*(?:[.]\d+)?(?![\w])", text
            ):
                out.append(m.group(0).replace(" ", ""))
            return out

        for cid in required:
            if cid not in const_values:
                fail(7, f"{cid} is not present in {const_path} (test is stale)")
                ac7 = False
                continue
            expected_text = const_values[cid]
            if cid not in entries:
                fail(7, f"{cid} missing from the Constants subsection")
                ac7 = False
                continue
            actual_text = entries[cid]
            expected_nums = numbers(expected_text)
            actual_nums = set(numbers(actual_text))
            missing = [n for n in expected_nums if n not in actual_nums]
            if missing:
                fail(
                    7,
                    f"{cid} mismatch: constants table says {expected_text!r} "
                    f"but wire.md says {actual_text!r} (missing value(s): {', '.join(missing)})",
                )
                ac7 = False
            elif not expected_nums and len(actual_text.strip()) < 6:
                fail(7, f"{cid} has no value in the Constants subsection")
                ac7 = False

if ac7:
    print(f"PASS AC7: all {len(required)} constants listed and matching {const_path}")

# ---------------------------------------------------------------------------
for ac, msg in warns:
    print(f"NOTE AC{ac}: {msg}")

if fails:
    for ac, msg in fails:
        print(f"FAIL AC{ac}: {msg}")
    print(f"{len(fails)} failure(s)")
    sys.exit(1)

print(f"OK: 7/7 acceptance criteria for 004-01")
PY
