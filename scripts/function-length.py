#!/usr/bin/env python3
"""Enforce the design owner's function-length rule (2026-11-14):

    "Every function that's over 100 lines of code must be split."

Scans Rust sources (*.rs) for `fn` items whose body contains more than
LIMIT lines of code, and reports each violation as
`file:function:line:count` (line = the line of the `fn` keyword, count =
code lines in the body).  Exits 1 if any function exceeds the limit,
0 otherwise.

Counting convention
-------------------
A function's body spans the lines strictly after the line holding the
body's opening `{`, through the line holding the closing `}`
(inclusive).  Within that range, a line counts as a *code line* unless
it is blank or comment-only (`//` line comments, `///`/`//!` doc
comments, and `/* */` block comments covering the whole line never
count).  The closing-brace line is part of the range and therefore
counts as one code line for any multi-line function; the 100-line limit
is applied to the count exactly as defined here.  String/char literal
content is code (it counts); comment content never counts.

The scanner is a small single-pass tokenizer: it tracks `//` line
comments, nested `/* */` block comments, `"..."` strings with `\\`
escapes, `'...'` char literals with escapes (distinguishing lifetimes
such as `'a` from char literals), and raw strings `r"..."`, `r#"..."#`
(also with a `b` prefix).  Braces inside strings, chars and comments are
ignored when matching a body, so a `"}"` inside a format string cannot
fool it.

Only `fn` items are measured.  Closures are not separate functions: they
live inside their enclosing function and count toward its length.

Known limitations: a lowercase `fn` *pointer type* written directly in a
`where` clause, or `fn` text inside a `macro_rules!` template, can
confuse the body locator (it may attach a following `{` as the body).
Neither occurs in this repository; revisit if it ever does.
"""

import sys
from pathlib import Path

LIMIT = 100
SKIP_DIRS = {".git", "target", "doc"}
IDENT = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"
)


def scan(src):
    """Tokenise `src` into (char, kind, line) triples, one per character.

    kind is "code", "string", "char" or "comment".  Newlines increment
    the line counter wherever they occur (including inside multi-line
    strings and block comments).
    """
    out = []
    i = 0
    n = len(src)
    line = 1

    def emit_span(start, end, kind):
        nonlocal i, line
        while start < end:
            c = src[start]
            out.append((c, kind, line))
            if c == "\n":
                line += 1
            start += 1
        i = end

    while i < n:
        ch = src[i]
        # line comment
        if ch == "/" and i + 1 < n and src[i + 1] == "/":
            j = i
            while j < n and src[j] != "\n":
                j += 1
            emit_span(i, j, "comment")
            continue
        # block comment (nestable, as in Rust)
        if ch == "/" and i + 1 < n and src[i + 1] == "*":
            depth = 1
            j = i + 2
            while j < n and depth:
                if src[j] == "/" and j + 1 < n and src[j + 1] == "*":
                    depth += 1
                    j += 2
                elif src[j] == "*" and j + 1 < n and src[j + 1] == "/":
                    depth -= 1
                    j += 2
                else:
                    j += 1
            emit_span(i, j, "comment")
            continue
        # raw string: r"..." / r#"..."#  (optionally byte: br"...", br#"..."#)
        if ch == "r" and i + 1 < n and src[i + 1] in "\"#":
            j = i + 1
            hashes = 0
            while j < n and src[j] == "#":
                hashes += 1
                j += 1
            if j < n and src[j] == '"':
                k = j + 1
                while k < n:
                    if src[k] == '"':
                        h = 0
                        while (
                            h < hashes
                            and k + 1 + h < n
                            and src[k + 1 + h] == "#"
                        ):
                            h += 1
                        if h == hashes:
                            break
                    k += 1
                end = min(k + 1 + hashes, n)  # closing quote + hashes
                emit_span(i, end, "string")
                continue
            # not a raw string after all: fall through, `r` is plain code
        # normal string
        if ch == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                elif src[j] == '"':
                    j += 1
                    break
                else:
                    j += 1
            emit_span(i, min(j, n), "string")
            continue
        # char literal vs lifetime
        if ch == "'":
            nxt = src[i + 1] if i + 1 < n else ""
            nxt2 = src[i + 2] if i + 2 < n else ""
            if nxt == "'" or nxt == "\\" or (nxt and nxt2 == "'"):
                j = i + 1
                while j < n:
                    if src[j] == "\\":
                        j += 2
                    elif src[j] == "'":
                        j += 1
                        break
                    else:
                        j += 1
                emit_span(i, min(j, n), "char")
                continue
            # lifetime ('a, 'static, '_): bare quote stays code
        if ch == "\n":
            line += 1
        out.append((ch, "code", line))
        i += 1
    return out


def analyze(src):
    """Return (functions, code_lines) for one source file.

    functions is a list of (name, fn_line, body_open_line, body_close_line)
    for every `fn` item with a body.
    """
    toks = scan(src)
    n = len(toks)

    code_lines = set()
    for ch, kind, ln in toks:
        if kind != "comment" and not ch.isspace():
            code_lines.add(ln)

    functions = []
    for idx in range(n):
        ch, kind, fn_line = toks[idx]
        if ch != "f" or kind != "code":
            continue
        if idx + 2 >= n:
            continue
        if toks[idx + 1][0] != "n" or toks[idx + 1][1] != "code":
            continue
        nxt_ch, nxt_kind, _ = toks[idx + 2]
        if nxt_ch in IDENT and nxt_kind == "code":
            continue  # part of a longer identifier (e.g. "format")
        if idx > 0 and toks[idx - 1][0] in IDENT and toks[idx - 1][1] == "code":
            continue  # part of a longer identifier (e.g. "unsafe_fn")

        # function name
        j = idx + 2
        while j < n:
            c, k, _ = toks[j]
            if k == "code" and (c.isalpha() or c == "_"):
                break
            j += 1
        start = j
        while j < n:
            c, k, _ = toks[j]
            if k == "code" and c in IDENT:
                j += 1
            else:
                break
        name = "".join(toks[p][0] for p in range(start, j))
        if name.startswith("r#"):
            name = name[2:]
        if not name:
            name = "<anon>"

        # locate the body: first { at zero paren/bracket/brace depth;
        # a ; at zero depth means a body-less signature (trait method,
        # fn pointer type) -- skip it.
        depth = 0
        body = None
        while j < n:
            c, k, _ = toks[j]
            if k != "code":
                j += 1
                continue
            if c == "{" and depth == 0:
                body = j
                break
            if c in "([{":
                depth += 1
            elif c in ")]}":
                if depth == 0:
                    break  # ran out of the enclosing construct
                depth -= 1
            elif c == ";" and depth == 0:
                break  # no body
            j += 1
        if body is None:
            continue

        # match the body's closing brace
        open_line = toks[body][2]
        d = 0
        close_line = open_line
        j = body
        while j < n:
            c, k, ln = toks[j]
            if k == "code":
                if c == "{":
                    d += 1
                elif c == "}":
                    d -= 1
                    if d == 0:
                        close_line = ln
                        break
            j += 1

        functions.append((name, fn_line, open_line, close_line))

    return functions, code_lines


def rust_files(root):
    for p in sorted(root.rglob("*.rs")):
        rel = p.relative_to(root)
        if set(rel.parts) & SKIP_DIRS:
            continue
        yield rel


def main():
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()
    violations = []
    total_fns = 0
    files = list(rust_files(root))
    for rel in files:
        try:
            src = (root / rel).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as exc:
            print(f"warning: cannot read {rel}: {exc}", file=sys.stderr)
            continue
        functions, code_lines = analyze(src)
        total_fns += len(functions)
        for name, fn_line, open_line, close_line in functions:
            count = sum(
                1
                for ln in range(open_line + 1, close_line + 1)
                if ln in code_lines
            )
            if count > LIMIT:
                violations.append((rel, name, fn_line, count))

    if violations:
        print(
            f"FAIL: {len(violations)} function(s) over {LIMIT} code lines:"
        )
        for rel, name, fn_line, count in violations:
            print(f"  {rel}:{name}:{fn_line}: {count} code lines")
        print(
            f"({total_fns} functions checked across {len(files)} files; "
            f"limit is {LIMIT} code lines)"
        )
        sys.exit(1)

    print(
        f"OK: {total_fns} functions checked across {len(files)} files, "
        f"none over {LIMIT} code lines"
    )
    sys.exit(0)


if __name__ == "__main__":
    main()
