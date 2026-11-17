//! The text model of the epic-002 bash scripts: a file as `grep`, `sed`
//! and `read` see it, and the bash, `tr` and `cut` string operations they
//! use. Each helper names the shell construct it reproduces; the module
//! `proof` compares each with the real tools' output in the committed
//! tables (`data/family002/corpus.tsv`, rows `H`, `HC` and `R`).

use super::translate::is_space;

/// A file as GNU `grep`, `sed` and `wc -l` see it: lines separated by
/// `\n`; a final line without a newline is still a line (for `grep` and
/// `sed`, not for `read`, see [`Lines::read_loop_from`]); a carriage
/// return stays in its line (no universal newlines: a CRLF file has lines
/// ending in `\r`).
#[derive(Debug, Clone)]
pub struct Lines<'a> {
    lines: Vec<&'a str>,
    final_newline: bool,
}

impl<'a> Lines<'a> {
    /// The lines of `text`.
    pub fn new(text: &'a str) -> Lines<'a> {
        let mut lines: Vec<&str> = text.split('\n').collect();
        let final_newline = text.ends_with('\n');
        if final_newline || text.is_empty() {
            lines.pop();
        }
        Lines {
            lines,
            final_newline,
        }
    }

    /// The number of lines `grep -n` and `sed` number (one more than
    /// `wc -l` when the last line has no newline).
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// True for an empty file.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// `wc -l`: the number of newline characters.
    pub fn wc_l(&self) -> usize {
        self.lines.len() - usize::from(!self.final_newline && !self.lines.is_empty())
    }

    /// Line `n` (1-based), if it exists.
    pub fn line(&self, n: usize) -> Option<&'a str> {
        n.checked_sub(1).and_then(|i| self.lines.get(i)).copied()
    }

    /// `$(sed -n "${n}p" file)`: line `n`, or the empty string past the end.
    pub fn sed_line(&self, n: usize) -> &'a str {
        self.line(n).unwrap_or("")
    }

    /// The line numbers `sed -n "${from},${to}p"` prints: from `from` to
    /// `to`, clipped at the end; when `to < from`, only line `from` (sed's
    /// rule for a second address before the first); nothing for line 0
    /// (sed refuses the address).
    pub fn sed_range(&self, from: usize, to: usize) -> std::ops::RangeInclusive<usize> {
        if from == 0 || from > self.len() {
            #[allow(clippy::reversed_empty_ranges)]
            return 1..=0;
        }
        from..=to.max(from).min(self.len())
    }

    /// `$(sed -n "${from},${to}p" file)`: those lines joined by newlines,
    /// trailing newlines removed (command substitution), so trailing empty
    /// lines disappear.
    pub fn sed_range_text(&self, from: usize, to: usize) -> String {
        let r = self.sed_range(from, to);
        let parts: Vec<&str> = r.map(|n| self.lines[n - 1]).collect();
        command_substitution(&parts.join("\n")).to_string()
    }

    /// The scripts' `line_range file from to`: nothing when `from > to`,
    /// else [`Lines::sed_range_text`].
    pub fn line_range(&self, from: usize, to: usize) -> String {
        if from > to {
            return String::new();
        }
        self.sed_range_text(from, to)
    }

    /// The lines `while IFS= read -r ln; do …; done < <(sed -n "${from},\$p" file)`
    /// gives the loop body: lines `from`.. the end, except that a last line
    /// without a newline is lost (`read` fails on it).
    pub fn read_loop_from(&self, from: usize) -> Vec<&'a str> {
        let r = self.sed_range(from, self.len());
        let last = *r.end();
        r.filter(|n| *n != last || self.final_newline)
            .map(|n| self.lines[n - 1])
            .collect()
    }

    /// The scripts' `heading_no file re`
    /// (`grep -Ein "$re" file | awk -F: '/^[0-9]+:#/ { print $1; exit }'`):
    /// the first line (1-based) that starts with `#` and matches `re`
    /// (case-insensitive, over the whole line, `#`s included). Not
    /// fence-aware: a `#` line inside a code block counts.
    pub fn heading_no(&self, re: &super::ere::Grep) -> Option<usize> {
        self.lines
            .iter()
            .position(|l| l.starts_with('#') && re.matches(l))
            .map(|i| i + 1)
    }
}

/// `$(…)`: the output with every trailing newline removed.
pub fn command_substitution(out: &str) -> &str {
    out.trim_end_matches('\n')
}

/// The lines `while IFS= read -r x; do …; done <<EOF` / `$S` / `EOF` gives the
/// loop body: `s` split on newlines (the here-document adds the final
/// one, so every piece is read; an empty `s` gives one empty line).
pub fn heredoc_lines(s: &str) -> Vec<&str> {
    s.split('\n').collect()
}

/// `s="${s#"${s%%[![:space:]]*}"}"`: leading `[[:space:]]` removed.
pub fn lstrip_space(s: &str) -> &str {
    s.trim_start_matches(is_space)
}

/// `s="${s%"${s##*[![:space:]]}"}"`: trailing `[[:space:]]` removed.
pub fn rstrip_space(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

/// `${s#prefix}` with a literal prefix: removed once when present.
pub fn strip_prefix_once<'s>(s: &'s str, prefix: &str) -> &'s str {
    s.strip_prefix(prefix).unwrap_or(s)
}

/// `${s%suffix}` with a literal suffix: removed once when present.
pub fn strip_suffix_once<'s>(s: &'s str, suffix: &str) -> &'s str {
    s.strip_suffix(suffix).unwrap_or(s)
}

/// `${s%%[!set]*}`: the leading run of characters from `set` (with
/// `${#…}`, its length in characters: `${line%%[! #]*}` of `## (d)` is
/// `## `, three).
pub fn leading_run<'s>(s: &'s str, set: &str) -> &'s str {
    let end = s.find(|c: char| !set.contains(c)).unwrap_or(s.len());
    &s[..end]
}

/// `tr '[:upper:]' '[:lower:]'`: GNU tr works on bytes, so only ASCII
/// letters change (`Δ` stays `Δ`).
pub fn tr_lower(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// `tr -d '[:space:]'`: the ASCII whitespace bytes removed (`tr` is
/// byte-based; no other repertoire character is a space).
pub fn tr_delete_space(s: &str) -> String {
    s.chars().filter(|c| !is_space(*c)).collect()
}

/// `printf '%s' "$s" | tr -s '[:space:]' '\n' | grep -c '[[:alnum:]]'`: the
/// whitespace-separated words that hold a letter or digit of the locale
/// (`²` alone is not one; `µ` is).
pub fn alnum_word_count(s: &str) -> usize {
    s.split(is_space)
        .filter(|w| {
            w.chars()
                .any(|c| super::translate::posix_class("alnum", c, false) == Some(true))
        })
        .count()
}

/// `printf '%s' "$s" | cut -c1-n`: GNU cut counts bytes (`-c` is `-b`), so a
/// multi-byte character can be cut in half; the result is the first `n`
/// bytes (lossy when a character is cut).
pub fn cut_bytes(s: &str, n: usize) -> String {
    let b = s.as_bytes();
    String::from_utf8_lossy(&b[..n.min(b.len())]).into_owned()
}

/// `… | sort -u | grep -c .`: the number of distinct non-empty strings.
/// (`sort -u` merges strings that collate equal in the locale; the
/// strings the scripts count, file:line pointers, are ASCII letters,
/// punctuation and digits, which collate distinctly: proven by the
/// fixture probe `ptr_distinct`.)
pub fn distinct_count<'s>(items: impl IntoIterator<Item = &'s str>) -> usize {
    let set: std::collections::BTreeSet<&str> =
        items.into_iter().filter(|s| !s.is_empty()).collect();
    set.len()
}

/// The integer `[ "$s" -le N ]` (and the other `test` comparisons) reads
/// from `s`, or `None` when bash says "integer expression expected" (the
/// test is then false): optional leading whitespace (`isspace`) and sign,
/// decimal digits (a leading zero is still decimal: `030` is 30), optional
/// trailing spaces or tabs, within 64 bits (`strtoimax` overflow is an
/// error). Bash 5.2 `legal_number`.
pub fn bash_test_int(s: &str) -> Option<i64> {
    let t = s.trim_start_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']);
    let (negative, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let end = t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len());
    let (digits, rest) = t.split_at(end);
    if digits.is_empty() || !rest.chars().all(|c| c == ' ' || c == '\t') {
        return None;
    }
    let n = crate::doccheck::pycompat::parse_int(digits).ok()?;
    i64::try_from(if negative { -n } else { n }).ok()
}
