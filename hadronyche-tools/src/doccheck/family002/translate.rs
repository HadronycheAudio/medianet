//! Translation of GNU grep/sed patterns (ERE, BRE, fixed strings) to the
//! library's regex syntax, for the two matchers GNU grep and sed combine:
//!
//! * the **DFA** (gnulib `dfa.c`), which decides whether a line matches when
//!   it supports the pattern. In a multibyte locale it supports a pattern
//!   only when the pattern has no word boundary (`\b \B \< \>`), no
//!   back-reference, no `\w \W \s \S`, and no bracket expression holding a
//!   class other than `[:digit:]`, a range other than digit-to-digit, a
//!   negation or a non-ASCII character (`dfa_supported`, and
//!   `parse_bracket_exp` returning `BACKREF`/`MBCSET`). Its ranges are code
//!   point ranges: `[0-9]` is the ten ASCII digits.
//! * the **regex matcher** (glibc `regcomp`/`re_search`), which decides the
//!   line for every other pattern, and gives the match positions (`grep -o`
//!   and every `sed s`). Its ranges follow the locale's collation: a
//!   superscript, subscript or vulgar fraction collates right after its
//!   base digit, so `[0-9]` also takes `¹ ² ³ ⁴ ⁵ ⁶ ₁ ₂ ₃ ⅓ ⅔` but not `⁹`
//!   (after `9`), and `[1-3]` takes `¹ ₁ ⅓ ² ₂ ⅔` but not `³ ₃`. Letter
//!   ranges are code point ranges in both (`[a-z]` is 26 letters).
//!
//! When the DFA does not support a pattern, grep and sed still run its
//! **superset** DFA first (`dfassbuild`): the pattern with every piece the
//! DFA cannot handle (a class bracket, `\w`, `\s`, `.`) replaced by "any
//! text" and word boundaries dropped, its digit ranges still code point
//! ranges. A line must pass the superset AND the regex matcher, so a digit
//! range must find an ASCII digit somewhere for the line to be selected:
//! `^[[:space:]]*([0-9]+[.)])[[:space:]]` (the 002-02 list entry) rejects
//! `²) item` but accepts `²) item - x` (the superset finds the `-`
//! alternative, the regex matcher reads `²` as a digit).
//!
//! Classes are the locale's in both (the DFA never handles one but
//! `[:digit:]`). Everything here is proven against the tools by
//! `family002::proof`.

use std::sync::OnceLock;

use crate::doccheck::repertoire;

/// The syntax a pattern is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Syntax {
    /// `grep -E`, `sed -E`.
    Ere,
    /// plain `grep` (basic regular expressions, GNU flavour).
    Bre,
    /// `grep -F`: the pattern is a literal string.
    Fixed,
}

/// Which matcher's range semantics to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// gnulib's DFA: code point ranges.
    Dfa,
    /// glibc's regex matcher: collation ranges.
    Regex,
}

/// A translated pattern, in the Rust syntax of the library's `Rx`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translated {
    /// The regex matcher's semantics (match positions; the line decision
    /// when the DFA does not support the pattern).
    pub regex: String,
    /// The DFA's semantics, when the DFA supports the pattern (it then
    /// decides whether a line matches).
    pub dfa: Option<String>,
    /// The superset DFA, when the DFA does not support the pattern and the
    /// pattern has a digit range (otherwise it accepts every line the regex
    /// matcher does): a line must match it too.
    pub superset: Option<String>,
    /// True when the pattern needs look-around (word boundaries): the
    /// sources then compile on `fancy-regex`, with [`Translated::relaxed`]
    /// as the prefilter.
    pub fancy: bool,
    /// The regex source with the word boundaries removed (it matches
    /// wherever the source does).
    pub relaxed: String,
}

/// One item of a bracket expression.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Char(char),
    Range(char, char),
    Class(String),
}

/// One element of a pattern, whatever its syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Lit(char),
    Any,
    Bracket {
        items: Vec<Item>,
        negated: bool,
    },
    /// `\w \W \s \S`: the class name and its negation.
    Escape {
        class: &'static str,
        negated: bool,
    },
    Open,
    Close,
    Alt,
    /// `*`, `+`, `?` or `{n,m}`, in Rust syntax.
    Quant(String),
    Start,
    End,
    /// `\b`, `\B`, `\<`, `\>`.
    Bound(char),
}

/// Translates `pattern` (in `syntax`, `-i` when `icase`).
pub fn translate(syntax: Syntax, icase: bool, pattern: &str) -> Result<Translated, String> {
    let toks = match syntax {
        Syntax::Fixed => pattern.chars().map(Tok::Lit).collect(),
        Syntax::Ere => ere_tokens(pattern)?,
        Syntax::Bre => bre_tokens(pattern)?,
    };
    check_quantifiers(&toks, pattern)?;
    for t in &toks {
        if let Tok::Bracket { items, .. } = t {
            for it in items {
                if let Item::Range(lo, hi) = it
                    && !(lo.is_ascii_digit() && hi.is_ascii_digit()
                        || lo.is_ascii_lowercase() && hi.is_ascii_lowercase()
                        || lo.is_ascii_uppercase() && hi.is_ascii_uppercase())
                {
                    return Err(format!(
                        "the range {lo}-{hi} in {pattern:?} is not supported (only digit or same-case letter ranges are proven)"
                    ));
                }
            }
        }
    }
    let fancy = toks.iter().any(|t| matches!(t, Tok::Bound(_)));
    let supported = dfa_supported(&toks);
    let digit_range = toks.iter().any(|t| match t {
        Tok::Bracket { items, .. } => items
            .iter()
            .any(|it| matches!(it, Item::Range(lo, _) if lo.is_ascii_digit())),
        _ => false,
    });
    Ok(Translated {
        regex: emit(&toks, icase, Engine::Regex, true),
        dfa: supported.then(|| emit(&toks, icase, Engine::Dfa, true)),
        superset: (!supported && digit_range).then(|| superset(&toks, icase)),
        fancy,
        relaxed: emit(&toks, icase, Engine::Regex, false),
    })
}

/// True when gnulib's DFA supports the pattern in a multibyte locale.
fn dfa_supported(toks: &[Tok]) -> bool {
    toks.iter().all(|t| match t {
        Tok::Bound(_) | Tok::Escape { .. } => false,
        Tok::Bracket { items, negated } => bracket_supported(items, *negated),
        _ => true,
    })
}

/// True when the DFA handles this bracket (multibyte locale): no negation,
/// only ASCII characters, digit ranges and `[:digit:]`.
fn bracket_supported(items: &[Item], negated: bool) -> bool {
    !negated
        && items.iter().all(|it| match it {
            Item::Char(c) => c.is_ascii(),
            Item::Range(lo, hi) => lo.is_ascii_digit() && hi.is_ascii_digit(),
            Item::Class(name) => name == "digit",
        })
}

/// The superset DFA's pattern: unsupported brackets, `\w \s` and `.` match
/// any text, word boundaries nothing; the rest as the DFA reads it.
fn superset(toks: &[Tok], icase: bool) -> String {
    let mut out = String::new();
    for t in toks {
        match t {
            Tok::Bracket { items, negated } if bracket_supported(items, *negated) => {
                let member = |c: char| bracket_member(items, c, icase, Engine::Dfa);
                out.push_str(&class_source(&member, false));
            }
            Tok::Bracket { .. } | Tok::Escape { .. } | Tok::Any => out.push_str(r"(?:[^\n]*)"),
            Tok::Bound(_) => {}
            other => out.push_str(&emit(
                std::slice::from_ref(other),
                icase,
                Engine::Dfa,
                false,
            )),
        }
    }
    out
}

/// A quantifier needs something to repeat (GNU reads a leading `*` in an
/// ERE as literal, which no script uses: refused).
fn check_quantifiers(toks: &[Tok], pattern: &str) -> Result<(), String> {
    for (i, t) in toks.iter().enumerate() {
        if let Tok::Quant(_) = t {
            let prev = i.checked_sub(1).map(|j| &toks[j]);
            if matches!(
                prev,
                None | Some(Tok::Open | Tok::Alt | Tok::Start | Tok::Quant(_) | Tok::Bound(_))
            ) {
                return Err(format!(
                    "a quantifier with nothing to repeat in {pattern:?} is not supported"
                ));
            }
        }
    }
    Ok(())
}

fn emit(toks: &[Tok], icase: bool, engine: Engine, bounds: bool) -> String {
    let w = class_source(&is_word, false);
    let mut out = String::new();
    for t in toks {
        match t {
            Tok::Lit(c) => out.push_str(&literal(*c, icase)),
            Tok::Any => out.push_str(r"[^\n]"),
            Tok::Bracket { items, negated } => {
                let member = |c: char| bracket_member(items, c, icase, engine);
                out.push_str(&class_source(&member, *negated));
            }
            Tok::Escape { class, negated } => {
                let member = |c: char| {
                    if *class == "word" {
                        is_word(c)
                    } else {
                        is_space(c)
                    }
                };
                out.push_str(&class_source(&member, *negated));
            }
            Tok::Open => out.push('('),
            Tok::Close => out.push(')'),
            Tok::Alt => out.push('|'),
            Tok::Quant(q) => out.push_str(q),
            Tok::Start => out.push('^'),
            Tok::End => out.push('$'),
            Tok::Bound(b) if bounds => out.push_str(&match b {
                'b' => format!("(?:(?<={w})(?!{w})|(?<!{w})(?={w}))"),
                'B' => format!("(?:(?<={w})(?={w})|(?<!{w})(?!{w}))"),
                '<' => format!("(?:(?<!{w})(?={w}))"),
                _ => format!("(?:(?<={w})(?!{w}))"),
            }),
            Tok::Bound(_) => {}
        }
    }
    out
}

fn literal(c: char, icase: bool) -> String {
    if icase && c.is_ascii_alphabetic() {
        format!("[{}{}]", c.to_ascii_lowercase(), c.to_ascii_uppercase())
    } else {
        regex::escape(c.encode_utf8(&mut [0; 4]))
    }
}

/// The repertoire minus newline: the characters a line can hold.
fn universe() -> &'static Vec<char> {
    static U: OnceLock<Vec<char>> = OnceLock::new();
    U.get_or_init(|| {
        repertoire::table()
            .keys()
            .copied()
            .filter(|c| *c != '\n')
            .collect()
    })
}

/// A Rust class holding exactly the repertoire characters `member` accepts
/// (negated: every other character), spelled out code point by code point
/// so that no Rust class semantics or case folding enters.
fn class_source(member: &dyn Fn(char) -> bool, negated: bool) -> String {
    let mut s = String::from(if negated { "[^" } else { "[" });
    let mut any = false;
    for c in universe().iter().copied().filter(|c| member(*c)) {
        s.push_str(&format!("\\x{{{:X}}}", u32::from(c)));
        any = true;
    }
    if !any {
        return if negated {
            r"[^\n]".into()
        } else {
            r"[^\x00-\x{10FFFF}]".into()
        };
    }
    s.push(']');
    s
}

fn bracket_member(items: &[Item], c: char, icase: bool, engine: Engine) -> bool {
    let base = |c: char| {
        items.iter().any(|it| match it {
            Item::Char(x) => *x == c,
            Item::Range(lo, hi) => in_range(*lo, *hi, c, engine),
            Item::Class(name) => posix_class(name, c, icase) == Some(true),
        })
    };
    base(c) || (icase && swap_case(c).is_some_and(base))
}

/// The repertoire characters that collate right after an ASCII digit (the
/// regex matcher's ranges), with that digit.
pub const DIGIT_VARIANTS: [(char, char); 11] = [
    ('¹', '1'),
    ('₁', '1'),
    ('⅓', '1'),
    ('²', '2'),
    ('₂', '2'),
    ('⅔', '2'),
    ('³', '3'),
    ('₃', '3'),
    ('⁴', '4'),
    ('⁵', '5'),
    ('⁶', '6'),
];

/// `c` in the range `lo-hi` for `engine` (digit or same-case ASCII letter
/// ranges; [`translate`] refuses others). `⁹` collates after `9` and is in
/// no digit range.
pub fn in_range(lo: char, hi: char, c: char, engine: Engine) -> bool {
    if lo <= c && c <= hi {
        return true;
    }
    engine == Engine::Regex
        && lo.is_ascii_digit()
        && DIGIT_VARIANTS
            .iter()
            .any(|(v, base)| *v == c && lo <= *base && *base < hi)
}

// -------------------------------------------------- the locale's classes

/// The letters of the repertoire outside ASCII, as the locale classes them.
const LOWER_NON_ASCII: &str = "µαπτ";
const UPPER_NON_ASCII: &str = "ΔΣ";

/// `[[:name:]]` membership of `c` (a repertoire character) in the scripts'
/// locale, with `-i` when `icase` (`[[:upper:]]` and `[[:lower:]]` then
/// take every letter); `None` for an unknown class name.
pub fn posix_class(name: &str, c: char, icase: bool) -> Option<bool> {
    let letter_lo = LOWER_NON_ASCII.contains(c);
    let letter_up = UPPER_NON_ASCII.contains(c);
    let letter = c.is_ascii_alphabetic() || letter_lo || letter_up;
    let other = !c.is_ascii() && !letter_lo && !letter_up;
    Some(match name {
        "alnum" => letter || c.is_ascii_digit(),
        "alpha" => letter,
        "digit" => c.is_ascii_digit(),
        "space" => matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r'),
        "upper" | "lower" if icase => letter,
        "upper" => c.is_ascii_uppercase() || letter_up,
        "lower" => c.is_ascii_lowercase() || letter_lo,
        "punct" => c.is_ascii_punctuation() || other,
        "blank" => matches!(c, ' ' | '\t'),
        "print" => (' '..='~').contains(&c) || !c.is_ascii(),
        "graph" => ('!'..='~').contains(&c) || !c.is_ascii(),
        "cntrl" => c.is_ascii_control(),
        "xdigit" => c.is_ascii_hexdigit(),
        _ => return None,
    })
}

/// `\w` (and the word class of `\b`): the locale's `[[:alnum:]_]`.
pub fn is_word(c: char) -> bool {
    c == '_' || posix_class("alnum", c, false) == Some(true)
}

/// `[[:space:]]` in the locale (grep, sed and bash agree).
pub fn is_space(c: char) -> bool {
    posix_class("space", c, false) == Some(true)
}

fn swap_case(c: char) -> Option<char> {
    if c.is_ascii_lowercase() {
        Some(c.to_ascii_uppercase())
    } else if c.is_ascii_uppercase() {
        Some(c.to_ascii_lowercase())
    } else {
        None
    }
}

// ------------------------------------------------------------- tokenizers

/// Parses a bracket expression starting after `[`; returns it and the
/// index after the closing `]`. A backslash is literal inside brackets.
fn bracket(p: &[char], mut i: usize, pattern: &str) -> Result<(Tok, usize), String> {
    let negated = p.get(i) == Some(&'^');
    if negated {
        i += 1;
    }
    let mut items = Vec::new();
    let mut first = true;
    loop {
        let Some(&c) = p.get(i) else {
            return Err(format!("unterminated bracket expression in {pattern:?}"));
        };
        if c == ']' && !first {
            return Ok((Tok::Bracket { items, negated }, i + 1));
        }
        first = false;
        if c == '[' && matches!(p.get(i + 1), Some(':' | '=' | '.')) {
            let kind = p[i + 1];
            let rest: String = p[i + 2..].iter().collect();
            let end = rest
                .find(&format!("{kind}]"))
                .ok_or_else(|| format!("unterminated [{kind} in {pattern:?}"))?;
            let name = rest[..end].to_string();
            if kind != ':' || posix_class(&name, 'a', false).is_none() {
                return Err(format!(
                    "[{kind}{name}{kind}] is not supported in {pattern:?}"
                ));
            }
            i += 2 + name.chars().count() + 2;
            items.push(Item::Class(name));
            continue;
        }
        if p.get(i + 1) == Some(&'-') && p.get(i + 2).is_some_and(|e| *e != ']') {
            items.push(Item::Range(c, p[i + 2]));
            i += 3;
            continue;
        }
        items.push(Item::Char(c));
        i += 1;
    }
}

/// The GNU escapes common to ERE and BRE; `None` when not one of them.
fn gnu_escape(c: char) -> Option<Tok> {
    Some(match c {
        'w' => Tok::Escape {
            class: "word",
            negated: false,
        },
        'W' => Tok::Escape {
            class: "word",
            negated: true,
        },
        's' => Tok::Escape {
            class: "space",
            negated: false,
        },
        'S' => Tok::Escape {
            class: "space",
            negated: true,
        },
        'b' | 'B' | '<' | '>' => Tok::Bound(c),
        _ => return None,
    })
}

/// An interval `{n}`, `{n,}`, `{,m}`, `{n,m}` starting after `{` and ending
/// at `close` (`}` or `\}`); returns the Rust quantifier and the index after.
fn interval(p: &[char], i: usize, close: &str, pattern: &str) -> Result<(Tok, usize), String> {
    let rest: String = p[i..].iter().collect();
    let end = rest
        .find(close)
        .ok_or_else(|| format!("unterminated interval in {pattern:?}"))?;
    let body = &rest[..end];
    let ok = !body.is_empty()
        && body.chars().all(|c| c.is_ascii_digit() || c == ',')
        && body.matches(',').count() <= 1
        && body != ",";
    if !ok {
        return Err(format!(
            "interval {{{body}}} is not supported in {pattern:?}"
        ));
    }
    let q = match body.strip_prefix(',') {
        Some(m) => format!("{{0,{m}}}"),
        None => format!("{{{body}}}"),
    };
    Ok((
        Tok::Quant(q),
        i + body.chars().count() + close.chars().count(),
    ))
}

fn ere_tokens(pattern: &str) -> Result<Vec<Tok>, String> {
    let p: Vec<char> = pattern.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < p.len() {
        let c = p[i];
        i += 1;
        let tok = match c {
            '\\' => {
                let Some(&n) = p.get(i) else {
                    return Err(format!("trailing backslash in {pattern:?}"));
                };
                i += 1;
                match gnu_escape(n) {
                    Some(t) => t,
                    None if ".[]()*+?{}|^$\\".contains(n) => Tok::Lit(n),
                    None => return Err(format!("\\{n} is not supported in {pattern:?}")),
                }
            }
            '[' => {
                let (t, next) = bracket(&p, i, pattern)?;
                i = next;
                t
            }
            '.' => Tok::Any,
            '(' => Tok::Open,
            ')' => Tok::Close,
            '|' => Tok::Alt,
            '*' | '+' | '?' => Tok::Quant(c.to_string()),
            '{' => {
                let (t, next) = interval(&p, i, "}", pattern)?;
                i = next;
                t
            }
            '^' => Tok::Start,
            '$' => Tok::End,
            _ => Tok::Lit(c),
        };
        out.push(tok);
    }
    balanced(&out, pattern)?;
    Ok(out)
}

fn balanced(toks: &[Tok], pattern: &str) -> Result<(), String> {
    let mut depth = 0i32;
    for t in toks {
        match t {
            Tok::Open => depth += 1,
            Tok::Close => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return Err(format!("unmatched ) in {pattern:?} is not supported"));
        }
    }
    if depth != 0 {
        return Err(format!("unmatched ( in {pattern:?}"));
    }
    Ok(())
}

fn bre_tokens(pattern: &str) -> Result<Vec<Tok>, String> {
    let p: Vec<char> = pattern.chars().collect();
    let mut out: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < p.len() {
        let c = p[i];
        i += 1;
        let at_start = matches!(out.last(), None | Some(Tok::Open | Tok::Alt | Tok::Start));
        let tok = match c {
            '\\' => {
                let Some(&n) = p.get(i) else {
                    return Err(format!("trailing backslash in {pattern:?}"));
                };
                i += 1;
                match (n, gnu_escape(n)) {
                    (_, Some(t)) => t,
                    ('(', _) => Tok::Open,
                    (')', _) => Tok::Close,
                    ('|', _) => Tok::Alt,
                    ('+' | '?', _) => Tok::Quant(n.to_string()),
                    ('{', _) => {
                        let (t, next) = interval(&p, i, "\\}", pattern)?;
                        i = next;
                        t
                    }
                    (n, None) if ".[]*^$\\".contains(n) => Tok::Lit(n),
                    (n, None) => return Err(format!("\\{n} is not supported in {pattern:?}")),
                }
            }
            '[' => {
                let (t, next) = bracket(&p, i, pattern)?;
                i = next;
                t
            }
            '.' => Tok::Any,
            '*' if at_start => Tok::Lit('*'),
            '*' => Tok::Quant("*".into()),
            '^' if matches!(out.last(), None | Some(Tok::Open | Tok::Alt)) => Tok::Start,
            '$' if i == p.len()
                || p[i..].starts_with(&['\\', ')'])
                || p[i..].starts_with(&['\\', '|']) =>
            {
                Tok::End
            }
            _ => Tok::Lit(c),
        };
        out.push(tok);
    }
    balanced(&out, pattern)?;
    Ok(out)
}
