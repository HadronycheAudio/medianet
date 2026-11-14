#!/usr/bin/env bash
# checks/006-01.sh — timeline spec skeleton and constants table.
#
# Ticket: doc/design/tickets/006/006-01-Timeline-spec-skeleton-and-constants.md
# Validates doc/design/spec/timeline.md against the ticket's acceptance
# criteria.  The values and statuses are checked against
# doc/design/02-constants.md, which wins on any difference.
#
# Unprivileged and read-only (C-146); no file is created or modified.

set -u
cd "$(dirname "$0")/.."

exec python3 - <<'PY'
import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path.cwd()
SPEC = ROOT / "doc" / "design" / "spec" / "timeline.md"
CONSTANTS = ROOT / "doc" / "design" / "02-constants.md"

TICKET_IDS = [
    "C-01", "C-02", "C-10", "C-18", "C-19", "C-30", "C-33", "C-37",
    "C-38", "C-43", "C-44", "C-47", "C-61", "C-64", "C-65", "C-67",
    "C-78", "C-80", "C-102", "C-105", "C-111", "C-113", "C-120",
    "C-121", "C-122", "C-124", "C-134", "C-135", "C-136", "C-137",
    "C-159", "C-141", "C-162", "C-169", "C-181", "C-184", "C-187",
    "C-188", "C-189", "C-195", "C-197", "C-200", "C-202", "C-205",
    "C-207", "C-209", "C-212",
]
TICKET_ID_SET = set(TICKET_IDS)

# Work step 1: (heading as written in the ticket, ticket ids that fill it,
# required heading-token groups; each group needs at least one alternative).
SECTIONS = [
    ("Domain line and position map", ["006-02"],
     [["domain"], ["line"], ["position"], ["map"]]),
    ("Epoch width and wrap", ["006-03"],
     [["epoch"], ["width"], ["wrap"]]),
    ("Sync reply and line reconstruction", ["006-04"],
     [["sync"], ["reply"], ["line"], ["reconstruction", "reconstruct"]]),
    ("Epoch-bump causes", ["006-05"],
     [["epoch"], ["bump"], ["cause"]]),
    ("Discontinuity event", ["006-06"],
     [["discontinuity"], ["event"]]),
    ("Handover without a bump", ["006-07"],
     [["handover"], ["without", "no"], ["bump"]]),
    ("Source-side switch procedure and session numbering", ["006-09"],
     [["source"], ["switch"], ["procedure"], ["session"], ["number"]]),
    ("Modules in Locking or holdover", ["006-10"],
     [["module"], ["lock"], ["holdover"]]),
    ("Consumer-side procedures", ["006-11"],
     [["consumer"], ["procedure"]]),
    ("Health of an input on another epoch", ["006-21"],
     [["health"], ["input"], ["other", "another"], ["epoch"]]),
    ("Announcement ordering and loss", ["006-12"],
     [["announcement"], ["order"], ["loss"]]),
    ("Sequence diagrams", ["006-13", "006-14", "006-15"],
     [["sequence"], ["diagram"]]),
    ("Scenarios", ["006-16", "006-17", "006-18"],
     [["scenario"]]),
    ("Regime stamp epoch field", ["006-19"],
     [["regime"], ["stamp"], ["epoch"], ["field"]]),
    ("Open items", ["006-20"],
     [["open"], ["item"]]),
]

STOPWORDS = set(
    "a an and the of in on or to for with as at by from is are be into "
    "than this that these those its it".split()
)

results = []  # (ac, ok, message)


def record(ac, ok, message):
    results.append((ac, ok, message))


def fail(ac, message):
    record(ac, False, message)


def norm_text(s):
    s = unicodedata.normalize("NFKC", s)
    for ch in ("\u00a0", "\u2007", "\u202f", "\u2009", "\u200a"):
        s = s.replace(ch, " ")
    s = s.replace("\u200b", "").replace("\ufeff", "")
    # typographic quotes and dashes are copy/paste noise, not values
    s = re.sub(r"[\u2010-\u2015\u2212]", "-", s)
    s = (s.replace("\u2018", "'").replace("\u2019", "'")
         .replace("\u201c", '"').replace("\u201d", '"'))
    return re.sub(r"\s+", " ", s).strip()


def norm_value(s):
    s = norm_text(s)
    s = s.replace("`", "").replace("**", "").replace("__", "")
    return s


def norm_status(s):
    s = norm_text(s).lower().replace("_", " ").replace("-", " ")
    return re.sub(r"\s+", " ", s).strip()


def split_row(line):
    """Split a Markdown table row on unescaped pipes."""
    s = line.strip()
    if not s.startswith("|"):
        return None
    if s.endswith("|") and not s.endswith("\\|"):
        s = s[:-1]
    s = s[1:]
    cells = []
    cur = []
    i = 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s) and s[i + 1] == "|":
            cur.append("|")
            i += 2
            continue
        if c == "|":
            cells.append("".join(cur).strip())
            cur = []
            i += 1
            continue
        cur.append(c)
        i += 1
    cells.append("".join(cur).strip())
    return cells


def is_separator(line):
    s = line.strip()
    if "-" not in s:
        return False
    return re.fullmatch(r"\|?[\s:|-]+\|?", s) is not None


def scan_fences_and_headings(lines):
    """Return headings, fence errors, indented-heading errors."""
    headings = []
    errors = []
    visible = [True] * len(lines)
    in_fence = False
    fence_char = None
    fence_len = 0
    fence_start = None

    for i, line in enumerate(lines, 1):
        m = re.match(r"^[ \t]*(`{3,}|~{3,})(.*)$", line)
        if m:
            marker = m.group(1)
            tail = m.group(2).strip()
            if not in_fence:
                in_fence = True
                fence_char = marker[0]
                fence_len = len(marker)
                fence_start = i
            elif marker[0] == fence_char and len(marker) >= fence_len and tail == "":
                in_fence = False
                fence_char = None
                fence_len = 0
                fence_start = None
            visible[i - 1] = False
            continue
        if in_fence:
            visible[i - 1] = False
            continue
        if re.match(r"^[ \t]+#{1,6}[ \t]+\S", line):
            errors.append(
                "line %d: heading is indented and does not start the line with '#': %s"
                % (i, line.strip())
            )
        hm = re.match(r"^(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$", line)
        if hm:
            headings.append((i, len(hm.group(1)), hm.group(2).strip()))

    if in_fence:
        errors.append(
            "line %d: unclosed code fence opened with %s"
            % (fence_start, fence_char * fence_len)
        )
    return headings, errors, visible


def parse_tables(lines, visible):
    """Return tables as lists of (line_no, cells); only real tables."""
    tables = []
    i = 0
    n = len(lines)
    while i < n:
        if visible[i] and lines[i].lstrip().startswith("|"):
            block = []
            j = i
            while j < n and visible[j] and lines[j].lstrip().startswith("|"):
                block.append((j + 1, split_row(lines[j])))
                j += 1
            if len(block) >= 2 and is_separator(lines[i + 1]):
                tables.append(block)
            i = j
        else:
            i += 1
    return tables


def parse_constants_source():
    """id -> {value, status, line} from doc/design/02-constants.md."""
    if not CONSTANTS.exists():
        return None
    lines = CONSTANTS.read_text(encoding="utf-8").splitlines()
    out = {}
    for i, line in enumerate(lines, 1):
        cells = split_row(line)
        if not cells:
            continue
        cid = cells[0].strip().upper()
        if not re.fullmatch(r"C-\d{2,3}", cid):
            continue
        # Main tables carry Status as the fourth cell.  The "Rules settled
        # after the ticket split" table has only ID | Rule | Value; its
        # preamble says those rules are decisions unless marked otherwise.
        if len(cells) >= 4:
            out[cid] = {
                "value": cells[2],
                "status": cells[3],
                "line": i,
            }
        elif len(cells) == 3:
            value = cells[2]
            status = (
                "pending simulation"
                if re.search(r"pending\s*[- ]?\s*simulation", value, re.IGNORECASE)
                else "decided"
            )
            out[cid] = {
                "value": value,
                "status": status,
                "line": i,
            }
    return out


def heading_tokens(text):
    words = re.findall(r"[a-z0-9]+", norm_text(text).lower())
    out = set()
    for w in words:
        if w in STOPWORDS:
            continue
        if len(w) > 4 and w.endswith("ies"):
            w = w[:-3] + "y"
        elif len(w) > 5 and w.endswith("ing"):
            w = w[:-3]
        elif len(w) > 4 and w.endswith("ed"):
            w = w[:-2]
        elif len(w) > 3 and w.endswith("s") and not w.endswith("ss"):
            w = w[:-1]
        out.add(w)
    return out


def matching_heading(headings, groups):
    for line_no, level, text in headings:
        if level < 2:
            continue
        toks = heading_tokens(text)
        if all(any(alt in toks for alt in group) for group in groups):
            return (line_no, level, text)
    return None


def section_body(lines, headings, heading):
    line_no, level, _ = heading
    start = line_no  # 0-based index of the line after the heading
    end = len(lines)
    for other_no, other_level, _ in headings:
        if other_no > line_no and other_level <= level:
            end = other_no - 1
            break
    return "\n".join(lines[start:end])


def header_index(header, names):
    """Index of a column whose normalized header is one of names/prefixes."""
    for idx, cell in enumerate(header):
        c = norm_text(cell).lower().rstrip(".:")
        if c in names:
            return idx
        if any(c.startswith(n + " ") or c.startswith(n + "(") for n in names):
            return idx
    return None


def ids_in(text):
    found = set()
    for m in re.finditer(r"006-(\d{2})", text):
        found.add("006-" + m.group(1))
    for m in re.finditer(
        r"006-(\d{2})\s*(?:\.\.+|[\-\u2013\u2014]|to|through)\s*(?:006-)?(\d{2})",
        text,
        re.IGNORECASE,
    ):
        a, b = int(m.group(1)), int(m.group(2))
        if a <= b and b - a <= 30:
            for k in range(a, b + 1):
                found.add("006-%02d" % k)
    return found


def one_sentence_wins(text):
    text = norm_text(text)
    sentences = re.split(r"(?<=[.!?])\s+", text)
    for sentence in sentences:
        low = sentence.lower()
        if re.search(r"\bwins?\b|\bprevails?\b|\bprecedence\b", low) and re.search(
            r"\bdiffer|\bconflict|\bdisagree|\bdiscrepan", low
        ):
            return sentence
    return None


def main():
    # ---- AC 1: file, constants table, values, statuses, precedence ----
    if not SPEC.exists():
        for ac in ("AC1", "AC2", "AC3", "AC4"):
            fail(ac, "doc/design/spec/timeline.md does not exist")
        report()
        return

    try:
        lines = SPEC.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        for ac in ("AC1", "AC2", "AC3", "AC4"):
            fail(ac, "cannot read doc/design/spec/timeline.md: %s" % exc)
        report()
        return

    headings, md_errors, visible = scan_fences_and_headings(lines)
    tables = parse_tables(lines, visible)

    source = parse_constants_source()
    if source is None:
        fail(
            "AC1",
            "doc/design/02-constants.md does not exist; cannot verify values",
        )
        source = {}
    else:
        missing_source = sorted(TICKET_ID_SET - set(source))
        if missing_source:
            fail(
                "AC1",
                "02-constants.md lacks ids required by the ticket: %s"
                % ", ".join(missing_source),
            )

    # Find the constants table: header has Id and Value, rows contain C- ids.
    const_table = None
    for table in tables:
        header = table[0][1]
        id_col = header_index(header, ("id", "constant id"))
        value_col = header_index(header, ("value",))
        if id_col is None or value_col is None:
            continue
        data = table[2:]
        if any(
            re.fullmatch(r"C-\d{2,3}", (row[1][id_col] or "").strip().upper())
            for row in data
            if id_col < len(row[1])
        ):
            const_table = table
            break

    if const_table is None:
        fail(
            "AC1",
            "no constants table found (expected a Markdown table with Id and "
            "Value columns and C-01 to C-212 rows)",
        )
    else:
        header = const_table[0][1]
        id_col = header_index(header, ("id", "constant id"))
        value_col = header_index(header, ("value",))
        status_col = header_index(header, ("status", "state"))

        if status_col is None:
            fail("AC1", "constants table has no status column")
        elif status_col >= len(header):
            fail("AC1", "constants table status column is malformed")

        table_start = const_table[0][0]
        table_end = const_table[-1][0]

        # -- exactly the 47 ids --
        rows = {}
        malformed = []
        for line_no, cells in const_table[2:]:
            if not any((c or "").strip() for c in cells):
                continue
            if id_col >= len(cells):
                malformed.append("line %d: row has no id cell" % line_no)
                continue
            cid = (cells[id_col] or "").strip().upper()
            if not re.fullmatch(r"C-\d{2,3}", cid):
                malformed.append(
                    "line %d: unexpected row id %r (the table must carry only "
                    "the ticket's C- ids)" % (line_no, cid)
                )
                continue
            if cid in rows:
                malformed.append("line %d: duplicate row %s" % (line_no, cid))
                continue
            rows[cid] = (line_no, cells)

        if malformed:
            fail("AC1", "; ".join(malformed[:5]))

        missing = sorted(TICKET_ID_SET - set(rows))
        extra = sorted(set(rows) - TICKET_ID_SET)
        if missing:
            fail("AC1", "constants table is missing ids: %s" % ", ".join(missing))
        if extra:
            fail(
                "AC1",
                "constants table has ids not in the ticket's 47 rows: %s"
                % ", ".join(extra),
            )
        if not missing and not extra and len(rows) != 47:
            fail("AC1", "constants table has %d rows; expected exactly 47" % len(rows))

        # -- values and statuses match 02-constants.md --
        value_mismatches = []
        status_mismatches = []
        for cid in TICKET_IDS:
            if cid not in rows:
                continue
            line_no, cells = rows[cid]
            if value_col >= len(cells):
                value_mismatches.append(
                    "line %d: %s has no value cell" % (line_no, cid)
                )
            elif cid in source:
                doc_value = norm_value(cells[value_col])
                ref_value = norm_value(source[cid]["value"])
                if doc_value != ref_value:
                    value_mismatches.append(
                        "%s at line %d: document has %r; 02-constants.md has %r"
                        % (cid, line_no, cells[value_col].strip(),
                           source[cid]["value"].strip())
                    )
            if status_col is not None and status_col < len(cells) and cid in source:
                doc_status = norm_status(cells[status_col])
                ref_status = norm_status(source[cid]["status"])
                if doc_status != ref_status:
                    status_mismatches.append(
                        "%s at line %d: document has %r; 02-constants.md has %r"
                        % (cid, line_no, cells[status_col].strip(),
                           source[cid]["status"].strip())
                    )

        if value_mismatches:
            fail("AC1", "value mismatch against 02-constants.md: "
                 + " | ".join(value_mismatches[:3])
                 + ("" if len(value_mismatches) <= 3
                    else " (+%d more)" % (len(value_mismatches) - 3)))
        if status_mismatches:
            fail("AC1", "status mismatch against 02-constants.md: "
                 + " | ".join(status_mismatches[:3])
                 + ("" if len(status_mismatches) <= 3
                    else " (+%d more)" % (len(status_mismatches) - 3)))

        for cid in ("C-61", "C-195"):
            if cid not in rows:
                continue
            status_col_ok = status_col is not None and status_col < len(rows[cid][1])
            status = norm_status(rows[cid][1][status_col]) if status_col_ok else ""
            if "pending" not in status or "simulation" not in status:
                fail(
                    "AC1",
                    "%s must be marked pending simulation (found %r)"
                    % (cid, status or "no status"),
                )

        # -- the constants table is the first section after the introduction --
        const_heading = None
        for line_no, level, text in headings:
            if line_no < table_start and (const_heading is None
                                          or line_no > const_heading[0]):
                const_heading = (line_no, level, text)
        if const_heading is None or const_heading[1] < 2:
            fail(
                "AC1",
                "the constants table is not an introduced section; it must be "
                "the first section after the introduction, under its own heading",
            )
        else:
            h2s = [h for h in headings if h[1] == 2]
            ci = next((i for i, h in enumerate(h2s)
                       if h[0] == const_heading[0]), None)
            before = h2s[:ci] if ci is not None else h2s
            intro_re = re.compile(
                r"\b(introduction|overview|scope|product frame|conventions?|"
                r"evidence tags?|purpose)\b",
                re.IGNORECASE,
            )
            bad_before = [h for h in before if not intro_re.search(h[2])]
            if ci is None:
                fail("AC1", "constants table is not under a level-2 heading")
            elif bad_before:
                names = ", ".join(repr(h[2]) for h in before) or "none"
                fail(
                    "AC1",
                    "the constants table is not the first section after the "
                    "introduction; headings before it: %s" % names,
                )

        # -- a sentence stating that the constants table wins on any difference --
        section_text = "\n".join(lines[:table_end])
        if const_heading is not None:
            section_end = len(lines)
            for line_no, level, _ in headings:
                if line_no > const_heading[0] and level <= const_heading[1]:
                    section_end = line_no - 1
                    break
            section_text = "\n".join(lines[:section_end])
        sentence = one_sentence_wins(section_text)
        if sentence is None:
            fail(
                "AC1",
                "no sentence states that the constants table wins on any "
                "difference (searched the introduction and constants section)",
            )

    # ---- AC 2: every section present, each placeholder names its ticket ----
    for title, expected_ids, groups in SECTIONS:
        heading = matching_heading(headings, groups)
        if heading is None:
            fail("AC2", "missing section heading: %s" % title)
            continue
        combined = heading[2] + "\n" + section_body(lines, headings, heading)
        if not combined.strip():
            fail("AC2", "section %r is empty" % title)
            continue
        found = ids_in(combined)
        missing = [i for i in expected_ids if i not in found]
        if missing:
            fail(
                "AC2",
                "section %r does not name the ticket id that fills it "
                "(missing %s; written in the section: %s)"
                % (title, ", ".join(missing),
                   ", ".join(sorted(found)) or "no 006-TT id"),
            )

    # ---- AC 3: retired POC behaviour note ----
    note_line = None
    note_heading = None
    for line_no, level, text in headings:
        if re.search(r"retired\s+(poc|proof[- ]of[- ]concept)\s+behaviou?r",
                     text, re.IGNORECASE):
            note_heading = (line_no, level, text)
            note_line = line_no
            break
    if note_line is None:
        for i, line in enumerate(lines, 1):
            if re.search(r"retired\s+(poc|proof[- ]of[- ]concept)\s+behaviou?r",
                         line, re.IGNORECASE):
                note_line = i
                break
    if note_line is None:
        fail("AC3", "no 'Retired POC behaviour' note found")
    else:
        if note_heading is not None:
            region = section_body(lines, headings, note_heading)
        else:
            end = min(len(lines), note_line + 60)
            for line_no, level, _ in headings:
                if line_no > note_line:
                    end = min(end, line_no - 1)
                    break
            region = "\n".join(lines[note_line - 1:end])
        if "[code]" not in norm_text(region):
            fail("AC3", "the retired POC behaviour note does not carry [code]")
        if not re.search(r"module\.rs\s*:\s*1630\s*[-\u2013\u2014]\s*1634",
                         region):
            fail(
                "AC3",
                "the retired POC behaviour note does not cite "
                "module.rs:1630-1634",
            )

    # ---- AC 4: Markdown parses cleanly ----
    for err in md_errors:
        fail("AC4", err)

    for table in tables:
        header = table[0][1]
        width = len(header)
        for line_no, cells in table[1:]:
            if len(cells) != width:
                fail(
                    "AC4",
                    "line %d: table row has %d columns; its table header has %d"
                    % (line_no, len(cells), width),
                )

    report()


def report():
    print("006-01: timeline spec skeleton and constants table")
    print("  spec: doc/design/spec/timeline.md")
    print("  constants: doc/design/02-constants.md")
    if not results:
        print("  OK: all checks passed")
        sys.exit(0)

    current = None
    for ac, ok, message in results:
        if ac != current:
            print("  %s" % ac)
            current = ac
        if ok:
            print("    OK   %s" % message)
        else:
            print("    FAIL %s" % message)

    failures = sum(1 for _, ok, _ in results if not ok)
    print("RESULT: FAIL (%d check%s failed)" % (failures,
                                                "" if failures == 1 else "s"))
    sys.exit(1)


if __name__ == "__main__":
    main()
PY
