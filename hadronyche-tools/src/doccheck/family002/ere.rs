//! GNU grep and GNU sed as the epic-002 scripts ran them (`grep -E`,
//! `grep` (BRE), `grep -F`, each with or without `-i`, as `-q`, `-c` or
//! `-o`; `sed -E 's/…/…/g'`), on the library's [`Rx`].
//!
//! What is reproduced, each point proven against the real tools by the
//! committed tables (`data/family002/`, module `proof`):
//!
//! * **one line at a time.** A pattern is matched against a line without its
//!   newline; nothing matches across lines; `^` and `$` are the line's ends.
//!   A carriage return is an ordinary character (a CRLF line ends in `\r`,
//!   so `x$` does not match `x\r`).
//! * **two matchers.** Whether a line matches is decided by gnulib's DFA
//!   when it supports the pattern, else by glibc's regex matcher; match
//!   positions (`-o`, `sed s`) always come from the regex matcher, and
//!   only on a line the decision accepted. The two differ on digit ranges
//!   (code points vs collation: see [`super::translate`]), so
//!   `grep -oE 'audio\.rs:[0-9]+'` prints `audio.rs:6²` for the line
//!   `audio.rs:6²` and nothing for `audio.rs:²`, and
//!   `sed -E 's/[0-9]+\.[0-9]+//g'` leaves `².5` alone but empties
//!   `1.5 ².5`.
//! * **bracket expressions, POSIX style**, in the scripts' locale: classes
//!   with the locale's membership on the repertoire (`µ Δ Σ α π τ` are
//!   letters; `²`, `—`, `≥` ... are punctuation, not word characters, unlike
//!   Python's and Rust's `\w`); a backslash inside brackets is literal
//!   (`[\.]` matches a backslash or a dot); a leading `]` is literal.
//! * **`-i`**: an ASCII letter matches both cases; no non-ASCII character of
//!   the repertoire has a case partner in the repertoire; `[[:upper:]]` and
//!   `[[:lower:]]` take every letter. The translation spells every case
//!   pair out (`[aA]`), so Rust's own Unicode case folding never applies.
//! * **GNU extensions** `\w \W \s \S` and `\b \B \< \>` (look-arounds over
//!   the locale's `[[:alnum:]_]`, on `fancy-regex`, with a plain
//!   prefilter); an escaped special character is literal. Anything else
//!   (`\d`, which GNU grep reads as a stray `d` with a warning;
//!   back-references; a leading `*` in an ERE; an interval that is not one;
//!   a range other than digit or same-case letter) is refused: the pattern
//!   records an engine failure, so the check turns red under `REGEX_LIMIT`
//!   instead of guessing (R19).
//! * **BRE** (plain `grep`): `\( \) \{ \} \| \+ \?` are the operators;
//!   `+ ? ( ) { } |` are literal; `*` at the start is literal; `^` and `$`
//!   are anchors only at the ends.
//! * **`-o` and `sed s///g`** iterate left to right, skipping empty matches
//!   (`-o`) or an empty match right after the previous match (sed).
//!   POSIX picks the longest match at the leftmost position, the engine the
//!   first by priority; they agree for every pattern in the tables (a
//!   pattern is proven before a port may use it: [`unproven_patterns`]).
//!
//! The semantics are defined on the repertoire only: a character outside it
//! never reaches a check ([`crate::doccheck::CheckSpec::evaluate`] turns the
//! verdict red under `REPERTOIRE`).

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use super::translate::{Syntax, translate};
use crate::doccheck::rx::Rx;

/// The two compiled matchers of one pattern.
struct Compiled {
    /// glibc's regex semantics: positions, and the decision when `dfa` is
    /// `None`.
    regex: Arc<Rx>,
    /// gnulib's DFA semantics, when the DFA supports the pattern.
    dfa: Option<Arc<Rx>>,
    /// The superset DFA a line must also pass when the DFA does not
    /// support the pattern (only kept when it can reject a line the regex
    /// matcher accepts: the pattern has a digit range).
    superset: Option<Arc<Rx>>,
    /// Set once the pattern was used as a line test / as `-o` (recorded in
    /// the registry then, so a port is held to the uses it makes).
    used_plain: std::sync::atomic::AtomicBool,
    used_only: std::sync::atomic::AtomicBool,
}

/// One regular expression as grep ran it: syntax, `-i`, and the pattern
/// text exactly as written in the script.
#[derive(Clone)]
pub struct Grep {
    syntax: Syntax,
    icase: bool,
    pattern: String,
    m: Arc<Compiled>,
}

impl std::fmt::Debug for Grep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.kind(false), self.pattern)
    }
}

impl Grep {
    /// `grep -E pattern`.
    pub fn ere(pattern: &str) -> Grep {
        Grep::new(Syntax::Ere, false, pattern)
    }
    /// `grep -Ei pattern`.
    pub fn ere_i(pattern: &str) -> Grep {
        Grep::new(Syntax::Ere, true, pattern)
    }
    /// `grep pattern` (BRE).
    pub fn bre(pattern: &str) -> Grep {
        Grep::new(Syntax::Bre, false, pattern)
    }
    /// `grep -i pattern` (BRE).
    pub fn bre_i(pattern: &str) -> Grep {
        Grep::new(Syntax::Bre, true, pattern)
    }
    /// `grep -F pattern`.
    pub fn fixed(pattern: &str) -> Grep {
        Grep::new(Syntax::Fixed, false, pattern)
    }
    /// `grep -Fi pattern`.
    pub fn fixed_i(pattern: &str) -> Grep {
        Grep::new(Syntax::Fixed, true, pattern)
    }

    /// The pattern in `syntax`, with `-i` when `icase`. Never panics: a
    /// pattern the translation refuses records an engine failure at each use.
    pub fn new(syntax: Syntax, icase: bool, pattern: &str) -> Grep {
        Grep {
            syntax,
            icase,
            pattern: pattern.to_string(),
            m: compiled(syntax, icase, pattern),
        }
    }

    /// Records the use (`-o` or not) the first time it happens.
    fn record(&self, only: bool) {
        use std::sync::atomic::Ordering;
        let flag = if only {
            &self.m.used_only
        } else {
            &self.m.used_plain
        };
        if !flag.swap(true, Ordering::Relaxed) {
            used(self.kind(only), &self.pattern);
        }
    }

    /// The table key of this matcher: `grep-E`, `grep-Gi`, `grep-Eo`, ...
    pub fn kind(&self, only: bool) -> String {
        let s = match self.syntax {
            Syntax::Ere => "E",
            Syntax::Bre => "G",
            Syntax::Fixed => "F",
        };
        format!(
            "grep-{s}{}{}",
            if self.icase { "i" } else { "" },
            if only { "o" } else { "" }
        )
    }

    /// The pattern as written.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// True when grep selects `line` (one line, no newline in it).
    pub fn matches(&self, line: &str) -> bool {
        self.record(false);
        self.decide(line)
    }

    fn decide(&self, line: &str) -> bool {
        match (&self.m.dfa, &self.m.superset) {
            (Some(dfa), _) => dfa.is_match(line),
            (None, Some(sup)) => sup.is_match(line) && self.m.regex.is_match(line),
            (None, None) => self.m.regex.is_match(line),
        }
    }

    /// `printf '%s\n' "$text" | grep -q`: true when any line of `text`
    /// matches.
    pub fn any_line(&self, text: &str) -> bool {
        text.split('\n').any(|l| self.matches(l))
    }

    /// `printf '%s\n' "$text" | grep -c`: the number of matching lines.
    pub fn count_lines(&self, text: &str) -> usize {
        text.split('\n').filter(|l| self.matches(l)).count()
    }

    /// `grep -o` on one line: nothing unless the line is selected; then the
    /// regex matcher's non-empty matches, left to right.
    pub fn only<'h>(&self, line: &'h str) -> Vec<&'h str> {
        self.record(true);
        if !self.decide(line) {
            return Vec::new();
        }
        self.m
            .regex
            .find_iter(line)
            .map(|m| m.as_str())
            .filter(|m| !m.is_empty())
            .collect()
    }

    /// `printf '%s\n' "$text" | grep -o`: every line's matches, in order.
    pub fn only_lines<'h>(&self, text: &'h str) -> Vec<&'h str> {
        text.split('\n').flat_map(|l| self.only(l)).collect()
    }
}

/// A `sed -E` script of `s/re/replacement/[g]` commands separated by `;`,
/// applied to one line (the scripts pipe one line at a time).
#[derive(Clone)]
pub struct Sed {
    script: String,
    commands: Vec<(Grep, String, bool)>,
    refused: Option<Arc<Rx>>,
}

impl std::fmt::Debug for Sed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sed -E {:?}", self.script)
    }
}

impl Sed {
    /// `sed -E script`. A command the parser refuses records an engine
    /// failure at each use (R19).
    pub fn ere(script: &str) -> Sed {
        used("sed-E".to_string(), script);
        let (commands, refused) = match parse_sed(script) {
            Ok(cmds) => (
                cmds.into_iter()
                    .map(|(re, rep, g)| (grep_quiet(&re), rep, g))
                    .collect(),
                None,
            ),
            Err(why) => (
                Vec::new(),
                Some(Arc::new(Rx::refused(&format!("sed -E {script:?}"), &why))),
            ),
        };
        Sed {
            script: script.to_string(),
            commands,
            refused,
        }
    }

    /// The script as written.
    pub fn script(&self) -> &str {
        &self.script
    }

    /// The line after every command. A command whose pattern the line does
    /// not match (the matcher's decision, as for grep) leaves it alone;
    /// otherwise the regex matcher's matches are replaced.
    pub fn apply(&self, line: &str) -> String {
        if let Some(r) = &self.refused {
            r.is_match(line);
            return line.to_string();
        }
        let mut text = line.to_string();
        for (g, rep, global) in &self.commands {
            if g.decide(&text) {
                text = substitute(&g.m.regex, rep, *global, &text);
            }
        }
        text
    }
}

/// A matcher for a sed command (not recorded as a grep pattern: the sed
/// script is what the tables prove).
fn grep_quiet(pattern: &str) -> Grep {
    Grep {
        syntax: Syntax::Ere,
        icase: false,
        pattern: pattern.to_string(),
        m: compiled(Syntax::Ere, false, pattern),
    }
}

/// GNU sed's `s`: matches left to right; an empty match right after the
/// previous match is skipped; `&` is the match, `\1`..`\9` the groups.
fn substitute(rx: &Rx, rep: &str, global: bool, hay: &str) -> String {
    let mut out = String::with_capacity(hay.len());
    let (mut last, mut prev_end) = (0, None);
    for c in rx.captures_iter(hay) {
        let Some(m) = c.get(0) else { continue };
        if m.start() == m.end() && prev_end == Some(m.start()) {
            continue;
        }
        out.push_str(&hay[last..m.start()]);
        let mut chars = rep.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '&' => out.push_str(m.as_str()),
                '\\' => match chars.next() {
                    Some(d @ '1'..='9') => {
                        let n = d as usize - '0' as usize;
                        if let Some(g) = c.get(n) {
                            out.push_str(g.as_str());
                        }
                    }
                    Some('n') => out.push('\n'),
                    Some(other) => out.push(other),
                    None => out.push('\\'),
                },
                other => out.push(other),
            }
        }
        last = m.end();
        prev_end = Some(m.end());
        if !global {
            break;
        }
    }
    out.push_str(&hay[last..]);
    out
}

/// Splits `s/a/b/g; s/c/d/` into commands.
fn parse_sed(script: &str) -> Result<Vec<(String, String, bool)>, String> {
    let mut out = Vec::new();
    let chars: Vec<char> = script.chars().collect();
    let mut i = 0;
    loop {
        while i < chars.len() && (chars[i] == ' ' || chars[i] == ';') {
            i += 1;
        }
        if i == chars.len() {
            return Ok(out);
        }
        if chars[i] != 's' || chars.get(i + 1) != Some(&'/') {
            return Err(format!("only s/re/rep/ commands are supported: {script:?}"));
        }
        i += 2;
        let mut parts = [String::new(), String::new()];
        for part in &mut parts {
            while i < chars.len() && chars[i] != '/' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    if chars[i + 1] != '/' {
                        part.push('\\');
                    }
                    part.push(chars[i + 1]);
                    i += 2;
                } else {
                    part.push(chars[i]);
                    i += 1;
                }
            }
            if i == chars.len() {
                return Err(format!("unterminated s command: {script:?}"));
            }
            i += 1;
        }
        let mut global = false;
        while i < chars.len() && chars[i] != ';' {
            match chars[i] {
                'g' => global = true,
                ' ' => {}
                f => return Err(format!("sed flag {f:?} is not supported")),
            }
            i += 1;
        }
        let [re, rep] = parts;
        out.push((re, rep, global));
    }
}

// ------------------------------------------------------------------ registry

static USED: OnceLock<Mutex<BTreeSet<(String, String)>>> = OnceLock::new();

pub(crate) fn used(kind: String, pattern: &str) {
    let set = USED.get_or_init(|| Mutex::new(BTreeSet::new()));
    let mut set = set.lock().expect("family002 registry");
    if set.len() < 4096 {
        set.insert((kind, pattern.to_string()));
    }
}

/// The committed list of proven (tool, pattern) pairs.
const PROVEN: &str = include_str!("../../../data/family002/patterns.tsv");

/// The (tool, pattern) pairs used in this process so far that the committed
/// tables do not prove. A port's test runs the check (and its denials) and
/// asserts this is empty: every pattern a port gives grep or sed has been
/// compared with the real tool.
pub fn unproven_patterns() -> Vec<(String, String)> {
    let proven: BTreeSet<(String, String)> = PROVEN
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut f = l.split('\t');
            // an awk regex is proven by its test, match and gsub rows
            let kind = f.next()?;
            let kind = if kind.starts_with("awk-") {
                "awk"
            } else {
                kind
            };
            Some((kind.to_string(), unescape_field(f.next()?)))
        })
        .collect();
    let set = USED.get_or_init(|| Mutex::new(BTreeSet::new()));
    let set = set.lock().expect("family002 registry");
    set.iter()
        .filter(|k| !proven.contains(k))
        .cloned()
        .collect()
}

/// Reverses the tables' escaping (`\\`, `\t`, `\n`, `\r`, `\xHH`) for a
/// text field.
pub(crate) fn unescape_field(s: &str) -> String {
    String::from_utf8_lossy(&unescape_bytes(s)).into_owned()
}

/// Reverses the tables' escaping into bytes.
pub(crate) fn unescape_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'\\' || i + 1 == b.len() {
            out.push(b[i]);
            i += 1;
            continue;
        }
        match b[i + 1] {
            b'\\' => out.push(b'\\'),
            b't' => out.push(b'\t'),
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b'x' if i + 3 < b.len() + 1 => {
                let hex = std::str::from_utf8(&b[i + 2..(i + 4).min(b.len())]).unwrap_or("");
                out.push(u8::from_str_radix(hex, 16).unwrap_or(b'?'));
                i += 2;
            }
            other => {
                out.push(b'\\');
                out.push(other);
            }
        }
        i += 2;
    }
    out
}

// --------------------------------------------------------------- compilation

type Key = (Syntax, bool, String);
static CACHE: OnceLock<Mutex<HashMap<Key, Arc<Compiled>>>> = OnceLock::new();

fn compiled(syntax: Syntax, icase: bool, pattern: &str) -> Arc<Compiled> {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (syntax, icase, pattern.to_string());
    if let Some(c) = cache.lock().expect("family002 cache").get(&key) {
        return c.clone();
    }
    let label = format!(
        "{} {pattern:?}",
        match (syntax, icase) {
            (Syntax::Ere, false) => "grep -E",
            (Syntax::Ere, true) => "grep -Ei",
            (Syntax::Bre, false) => "grep",
            (Syntax::Bre, true) => "grep -i",
            (Syntax::Fixed, false) => "grep -F",
            (Syntax::Fixed, true) => "grep -Fi",
        }
    );
    let c = Arc::new(match translate(syntax, icase, pattern) {
        Ok(t) => {
            let pre = t.fancy.then_some(t.relaxed.as_str());
            Compiled {
                regex: Arc::new(Rx::new(&label, &t.regex, pre)),
                dfa: t.dfa.map(|d| Arc::new(Rx::new(&label, &d, None))),
                superset: t.superset.map(|d| Arc::new(Rx::new(&label, &d, None))),
                used_plain: Default::default(),
                used_only: Default::default(),
            }
        }
        Err(why) => Compiled {
            regex: Arc::new(Rx::refused(&label, &why)),
            dfa: None,
            superset: None,
            used_plain: Default::default(),
            used_only: Default::default(),
        },
    });
    let mut cache = cache.lock().expect("family002 cache");
    if cache.len() < 4096 {
        cache.insert(key, c.clone());
    }
    c
}
