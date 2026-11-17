//! awk as the epic-002 scripts ran it: mawk 1.3.4, which is BYTE-based in
//! every locale. A record is a byte string; `length`, `substr`, `index`,
//! `RSTART` and `RLENGTH` count bytes; `tolower`/`toupper` change ASCII
//! letters only; a regex matches bytes, so `.` is one byte, a class
//! (`[[:alnum:]]`) holds ASCII bytes only, and a multi-byte character
//! written inside brackets stands for its bytes one by one (`[×-]` holds
//! the bytes `C3` and `97`, so `÷`, whose first byte is also `C3`, is in
//! it when a one-byte `substr` is tested).
//!
//! [`AwkRe`] compiles an awk regex (a `/…/` literal, or a string used as a
//! dynamic regex: write it as the string's VALUE, after [`v_assign`] for a
//! `-v` assignment) to the `regex` crate's byte engine. `match` reports the
//! leftmost match; mawk's is the leftmost-longest, the engine's the
//! leftmost by priority, and they agree on every pattern of the tables
//! (`data/family002/corpus.tsv`, rows `AM`, `AG`, `AT`), which a port's
//! patterns must belong to ([`super::unproven_patterns`]).
//!
//! A pattern the translation does not support records an engine failure at
//! each use (R19): the check turns red under `REGEX_LIMIT`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::doccheck::rx::Rx;

/// A compiled awk regex.
#[derive(Clone)]
pub struct AwkRe {
    pattern: String,
    re: Result<regex::bytes::Regex, Arc<Rx>>,
}

impl std::fmt::Debug for AwkRe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "awk /{}/", self.pattern)
    }
}

impl AwkRe {
    /// The awk regex `pattern` (the text between the slashes of a literal,
    /// or the value of a dynamic regex string).
    pub fn new(pattern: &str) -> AwkRe {
        type Cache = Mutex<HashMap<String, AwkRe>>;
        static CACHE: OnceLock<Cache> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(re) = cache.lock().expect("awk cache").get(pattern) {
            return re.clone();
        }
        super::ere::used("awk".to_string(), pattern);
        let label = format!("awk /{pattern}/");
        let re = translate(pattern.as_bytes())
            .and_then(|src| regex::bytes::Regex::new(&src).map_err(|e| e.to_string()))
            .map_err(|why| Arc::new(Rx::refused(&label, &why)));
        let re = AwkRe {
            pattern: pattern.to_string(),
            re,
        };
        let mut cache = cache.lock().expect("awk cache");
        if cache.len() < 4096 {
            cache.insert(pattern.to_string(), re.clone());
        }
        re
    }

    fn engine(&self) -> Option<&regex::bytes::Regex> {
        match &self.re {
            Ok(r) => Some(r),
            Err(refused) => {
                refused.is_match("");
                None
            }
        }
    }

    /// `s ~ re`.
    pub fn test(&self, s: &[u8]) -> bool {
        self.engine().is_some_and(|r| r.is_match(s))
    }

    /// `match(s, re)`: `Some((RSTART, RLENGTH))` (1-based, in bytes), or
    /// `None` (RSTART 0, RLENGTH -1).
    pub fn find(&self, s: &[u8]) -> Option<(usize, usize)> {
        let m = self.engine()?.find(s)?;
        Some((m.start() + 1, m.end() - m.start()))
    }

    /// `gsub(re, repl, s)` (`global`) or `sub(re, repl, s)`: the new value
    /// of `s`. `&` in `repl` is the match, `\&` a literal `&`. After an
    /// empty match one byte is copied before the next try; an empty match
    /// right after a match is not replaced (mawk).
    pub fn replace(&self, s: &[u8], repl: &[u8], global: bool) -> Vec<u8> {
        let Some(r) = self.engine() else {
            return s.to_vec();
        };
        let mut out = Vec::with_capacity(s.len());
        let (mut pos, mut prev_end) = (0, None);
        while pos <= s.len() {
            let Some(m) = r.find_at(s, pos) else { break };
            if m.start() == m.end() && prev_end == Some(m.start()) {
                if m.start() < s.len() {
                    out.push(s[m.start()]);
                }
                pos = m.start() + 1;
                prev_end = None;
                continue;
            }
            out.extend_from_slice(&s[pos..m.start()]);
            expand(repl, &s[m.start()..m.end()], &mut out);
            prev_end = Some(m.end());
            if m.start() == m.end() {
                if m.start() < s.len() {
                    out.push(s[m.start()]);
                }
                pos = m.end() + 1;
            } else {
                pos = m.end();
            }
            if !global {
                break;
            }
        }
        if pos < s.len() {
            out.extend_from_slice(&s[pos..]);
        }
        out
    }
}

fn expand(repl: &[u8], matched: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < repl.len() {
        match repl[i] {
            b'\\' if repl.get(i + 1) == Some(&b'&') => {
                out.push(b'&');
                i += 2;
            }
            b'&' => {
                out.extend_from_slice(matched);
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
}

/// `substr(s, m, n)` (bytes, POSIX: the bytes at positions `m` to
/// `m + n - 1` that exist; `n = None` is "to the end").
pub fn substr(s: &[u8], m: i64, n: Option<i64>) -> &[u8] {
    let len = s.len() as i64;
    let start = m.max(1);
    let end = match n {
        Some(n) => (m.saturating_add(n) - 1).min(len),
        None => len,
    };
    if start > end || start > len {
        return &[];
    }
    &s[(start - 1) as usize..end as usize]
}

/// `tolower(s)`: ASCII letters only.
pub fn tolower(s: &[u8]) -> Vec<u8> {
    s.to_ascii_lowercase()
}

/// The value of a `-v name="$value"` assignment: mawk turns the escapes it
/// knows (`\\`, `\"`, `\a \b \f \n \r \t \v`) into their characters and
/// keeps any other backslash (`Q32\.32` and `\/` stay as they are).
pub fn v_assign(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let mapped = match chars.peek() {
            Some('\\') => Some('\\'),
            Some('"') => Some('"'),
            Some('a') => Some('\x07'),
            Some('b') => Some('\x08'),
            Some('f') => Some('\x0c'),
            Some('n') => Some('\n'),
            Some('r') => Some('\r'),
            Some('t') => Some('\t'),
            Some('v') => Some('\x0b'),
            _ => None,
        };
        match mapped {
            Some(m) => {
                out.push(m);
                chars.next();
            }
            None => out.push('\\'),
        }
    }
    out
}

// --------------------------------------------------------------- translation

/// The `regex` crate (bytes, Unicode off) source of an awk ERE.
fn translate(p: &[u8]) -> Result<String, String> {
    let mut out = String::from("(?-u)");
    let mut i = 0;
    let mut depth = 0i32;
    while i < p.len() {
        let c = p[i];
        i += 1;
        match c {
            b'\\' => {
                let n = *p.get(i).ok_or("trailing backslash")?;
                i += 1;
                let b = match n {
                    b't' => b'\t',
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b'/' | b'.' | b'[' | b']' | b'(' | b')' | b'*' | b'+' | b'?' | b'{' | b'}'
                    | b'|' | b'^' | b'$' | b'\\' | b'"' | b'-' => n,
                    other => return Err(format!("\\{} is not supported in awk", other as char)),
                };
                out.push_str(&byte(b));
            }
            b'[' => {
                let (class, next) = bracket(p, i)?;
                out.push_str(&class);
                i = next;
            }
            b'.' => out.push_str(r"[^\n]"),
            b'(' => {
                depth += 1;
                out.push('(');
            }
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return Err("unmatched )".into());
                }
                out.push(')');
            }
            b'|' | b'*' | b'+' | b'?' | b'^' | b'$' => out.push(c as char),
            b'{' => {
                let close = p[i..]
                    .iter()
                    .position(|b| *b == b'}')
                    .ok_or("unterminated interval")?;
                let body = std::str::from_utf8(&p[i..i + close]).map_err(|e| e.to_string())?;
                if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == ',') {
                    return Err(format!("interval {{{body}}} is not supported"));
                }
                out.push_str(&format!("{{{body}}}"));
                i += close + 1;
            }
            _ => out.push_str(&byte(c)),
        }
    }
    if depth != 0 {
        return Err("unmatched (".into());
    }
    Ok(out)
}

/// The byte an escape `\c` stands for in a mawk regex.
fn unescape(c: u8) -> u8 {
    match c {
        b't' => b'\t',
        b'n' => b'\n',
        b'r' => b'\r',
        other => other,
    }
}

fn byte(b: u8) -> String {
    format!("\\x{b:02X}")
}

/// A bracket expression after `[`, on bytes: classes are ASCII.
fn bracket(p: &[u8], mut i: usize) -> Result<(String, usize), String> {
    let negated = p.get(i) == Some(&b'^');
    if negated {
        i += 1;
    }
    let mut set = [false; 256];
    let mut first = true;
    loop {
        let c = *p.get(i).ok_or("unterminated bracket expression")?;
        if c == b']' && !first {
            i += 1;
            break;
        }
        first = false;
        if c == b'[' && p.get(i + 1) == Some(&b':') {
            let rest = &p[i + 2..];
            let end = rest
                .windows(2)
                .position(|w| w == b":]")
                .ok_or("unterminated [:")?;
            let name = std::str::from_utf8(&rest[..end]).map_err(|e| e.to_string())?;
            for b in 0u8..=127 {
                set[b as usize] |=
                    ascii_class(name, b).ok_or(format!("[:{name}:] is not supported"))?;
            }
            i += 2 + end + 2;
            continue;
        }
        let (lo, next) = if c == b'\\' && i + 1 < p.len() {
            (unescape(p[i + 1]), i + 2)
        } else {
            (c, i + 1)
        };
        if p.get(next) == Some(&b'-') && p.get(next + 1).is_some_and(|e| *e != b']') {
            let hi = p[next + 1];
            for b in lo..=hi {
                set[b as usize] = true;
            }
            i = next + 2;
        } else {
            set[lo as usize] = true;
            i = next;
        }
    }
    let mut s = String::from(if negated { "[^" } else { "[" });
    let members: Vec<usize> = (0..256).filter(|b| set[*b]).collect();
    if members.is_empty() {
        return Ok((
            if negated {
                r"[\x00-\xFF]".into()
            } else {
                r"[^\x00-\xFF]".into()
            },
            i,
        ));
    }
    for b in members {
        s.push_str(&byte(b as u8));
    }
    s.push(']');
    Ok((s, i))
}

fn ascii_class(name: &str, b: u8) -> Option<bool> {
    Some(match name {
        "alnum" => b.is_ascii_alphanumeric(),
        "alpha" => b.is_ascii_alphabetic(),
        "digit" => b.is_ascii_digit(),
        "space" => matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'),
        "upper" => b.is_ascii_uppercase(),
        "lower" => b.is_ascii_lowercase(),
        "punct" => b.is_ascii_punctuation(),
        "blank" => matches!(b, b' ' | b'\t'),
        "print" => (0x20..=0x7e).contains(&b),
        "graph" => (0x21..=0x7e).contains(&b),
        "cntrl" => b.is_ascii_control(),
        "xdigit" => b.is_ascii_hexdigit(),
        _ => return None,
    })
}
