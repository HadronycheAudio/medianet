//! The criteria of scripts that print UNLABELLED failures: the epic-002
//! bash scripts write `bad() { printf 'FAIL: %s\n' "$*"; }`, so a failure
//! reads `FAIL: sub-section is too long (52 non-empty lines, …)` with no
//! criterion in it, and [`super::script::verdict`] alone reads none (a red
//! script would read as a crash).
//!
//! The criterion is recovered from the script's own source:
//!
//! * every `bad …` call site is a template: its words as the shell joins
//!   them (`"$*"`), literal text kept, every expansion (`$x`, `${…}`,
//!   `$(…)`, `$((…))`, backquotes) matching any text;
//! * its criterion is the banner the site sits under: the comment line
//!   after a `# ----` rule, up to ` — ` (`# AC 1 — size and pointers` gives
//!   `AC1`, `# Work step 2 — …` gives `Work step 2`), or its first word when
//!   it has no dash (`# Locate section (d) …` gives `Locate`, `# Helpers`
//!   gives `Helpers`); a site before any banner gets `preamble`;
//! * a `FAIL: ` message takes the criterion of the template that matches it
//!   whole with the most literal characters (a multi-line message is read
//!   with its continuation lines). A message no template matches, or that
//!   two templates of different criteria match equally well, is UNREAD: the
//!   verdict is then a crash (never matched to a port), so a script's
//!   failure can never be silently dropped.
//!
//! The ports of these scripts name their criteria the same way.
//!
//! **Opt-in, per port** (coordinator's ruling): the reader applies only to
//! the ports listed in [`SELECTED`]; every other port's script is read by
//! the other readers alone, unchanged. One reader per port: a port listed
//! here selects no other unlabelled-line reader. The list is interim:
//! when `m/doccheck-lib` gives a port a single explicit reader setting,
//! each listed port sets it there and this list goes.

use std::collections::BTreeSet;

/// The ports that select this reader: the bash scripts of epic 002 ported
/// by WP1.
pub const SELECTED: &[&str] = &["002-02", "002-03", "002-04"];

/// True when the port `id` selected this reader.
pub fn selected(id: &str) -> bool {
    SELECTED.contains(&id)
}

/// One `bad` call site.
#[derive(Debug, Clone)]
pub struct Site {
    /// The criterion (the banner above it).
    pub criterion: String,
    /// The template as a whole-message regex.
    pub regex: regex::Regex,
    /// How many literal characters the template has (specificity).
    pub literal: usize,
    /// The script line (1-based), for messages.
    pub line: usize,
}

/// The criterion a banner title names.
pub fn banner_criterion(title: &str) -> String {
    let head = match title.split_once(" — ") {
        Some((h, _)) => h.trim().to_string(),
        None => title.split_whitespace().next().unwrap_or("").to_string(),
    };
    match head.strip_prefix("AC") {
        Some(n)
            if n.trim_start_matches([' ', '-'])
                .chars()
                .all(|c| c.is_ascii_digit())
                && !n.trim().is_empty() =>
        {
            format!("AC{}", n.trim_start_matches([' ', '-']))
        }
        _ => head,
    }
}

/// Every `bad` call site of `script`, with its criterion.
pub fn sites(script: &str) -> Vec<Site> {
    let lines: Vec<&str> = script.lines().collect();
    let mut criterion = "preamble".to_string();
    let mut out = Vec::new();
    let rule = regex::Regex::new(r"^#\s*-{5,}\s*$").expect("literal");
    let call = regex::Regex::new(r"(?:^|[\s;&|({])bad\s+").expect("literal");
    for (i, line) in lines.iter().enumerate() {
        if rule.is_match(line)
            && let Some(next) = lines.get(i + 1)
            && let Some(title) = next.strip_prefix('#')
            && !rule.is_match(next)
            && !title.trim().is_empty()
        {
            criterion = banner_criterion(title.trim());
        }
        let code = line.trim_start();
        if code.starts_with('#') || code.starts_with("bad()") {
            continue;
        }
        for m in call.find_iter(line) {
            let args = &line[m.end()..];
            if let Some((source, literal)) = template(args)
                && let Ok(regex) = regex::Regex::new(&format!("^(?s:{source})$"))
            {
                out.push(Site {
                    criterion: criterion.clone(),
                    regex,
                    literal,
                    line: i + 1,
                });
            }
        }
    }
    out
}

/// One piece of a template.
enum Piece {
    Lit(char),
    Wild,
}

/// The regex of the words `args` starts with (up to the end of the
/// command), and its literal character count.
fn template(args: &str) -> Option<(String, usize)> {
    let c: Vec<char> = args.chars().collect();
    let mut pieces = Vec::new();
    let mut i = 0;
    let mut words = 0;
    loop {
        while i < c.len() && (c[i] == ' ' || c[i] == '\t') {
            i += 1;
        }
        if i >= c.len() || matches!(c[i], ';' | '&' | '|' | ')' | '#' | '\n') {
            break;
        }
        if words > 0 {
            pieces.push(Piece::Lit(' '));
        }
        i = word(&c, i, &mut pieces)?;
        words += 1;
    }
    if words == 0 {
        return None;
    }
    let mut source = String::new();
    let mut literal = 0;
    let mut last_wild = false;
    for p in pieces {
        match p {
            Piece::Lit(ch) => {
                source.push_str(&regex::escape(ch.encode_utf8(&mut [0; 4])));
                literal += 1;
                last_wild = false;
            }
            Piece::Wild if !last_wild => {
                source.push_str(".*");
                last_wild = true;
            }
            Piece::Wild => {}
        }
    }
    Some((source, literal))
}

/// Reads one shell word from `c[i..]` into `out`; returns the index after.
fn word(c: &[char], mut i: usize, out: &mut Vec<Piece>) -> Option<usize> {
    while i < c.len() && !matches!(c[i], ' ' | '\t' | ';' | '&' | '|' | ')') {
        match c[i] {
            '"' => {
                i += 1;
                while i < c.len() && c[i] != '"' {
                    i = dq_char(c, i, out)?;
                }
                i += 1;
            }
            '\'' => {
                i += 1;
                while i < c.len() && c[i] != '\'' {
                    out.push(Piece::Lit(c[i]));
                    i += 1;
                }
                i += 1;
            }
            '\\' if i + 1 < c.len() => {
                out.push(Piece::Lit(c[i + 1]));
                i += 2;
            }
            '$' | '`' => i = expansion(c, i, out)?,
            ch => {
                out.push(Piece::Lit(ch));
                i += 1;
            }
        }
    }
    Some(i)
}

/// One character (or expansion) inside double quotes.
fn dq_char(c: &[char], i: usize, out: &mut Vec<Piece>) -> Option<usize> {
    match c[i] {
        '\\' if i + 1 < c.len() && matches!(c[i + 1], '"' | '\\' | '$' | '`') => {
            out.push(Piece::Lit(c[i + 1]));
            Some(i + 2)
        }
        '$' | '`' => expansion(c, i, out),
        ch => {
            out.push(Piece::Lit(ch));
            Some(i + 1)
        }
    }
}

/// An expansion at `c[i]` (`$…` or a backquote); pushes a wildcard.
fn expansion(c: &[char], i: usize, out: &mut Vec<Piece>) -> Option<usize> {
    if c[i] == '`' {
        let end = c[i + 1..].iter().position(|ch| *ch == '`')? + i + 2;
        out.push(Piece::Wild);
        return Some(end);
    }
    let next = c.get(i + 1).copied();
    let end = match next {
        Some('{') => closing(c, i + 1, '{', '}')?,
        Some('(') => closing(c, i + 1, '(', ')')?,
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
            let mut j = i + 1;
            while j < c.len() && (c[j].is_ascii_alphanumeric() || c[j] == '_') {
                j += 1;
            }
            j
        }
        Some(ch) if ch.is_ascii_digit() || "*@#?$!-".contains(ch) => i + 2,
        _ => {
            out.push(Piece::Lit('$'));
            return Some(i + 1);
        }
    };
    out.push(Piece::Wild);
    Some(end)
}

/// The index after the bracket closing the one at `c[open]`, skipping quoted
/// text and nested brackets.
fn closing(c: &[char], open: usize, l: char, r: char) -> Option<usize> {
    let mut depth = 0;
    let mut i = open;
    while i < c.len() {
        match c[i] {
            '\\' => i += 1,
            '\'' if l == '(' => i += c[i + 1..].iter().position(|ch| *ch == '\'')? + 1,
            '"' => {
                i += 1;
                while i < c.len() && c[i] != '"' {
                    if c[i] == '\\' {
                        i += 1;
                    } else if c[i] == '$' && c.get(i + 1) == Some(&'(') {
                        i = closing(c, i + 1, '(', ')')? - 1;
                    }
                    i += 1;
                }
            }
            ch if ch == l => depth += 1,
            ch if ch == r => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The criteria of the `FAIL: message` lines of `output`, read against the
/// `bad` call sites of `script`. `Err` lists the messages no site reads
/// (or that sites of different criteria read equally well).
pub fn unlabelled_failures(script: &str, lines: &[&str]) -> Result<BTreeSet<String>, Vec<String>> {
    let sites = sites(script);
    let mut out = BTreeSet::new();
    let mut unread = Vec::new();
    for (k, line) in lines.iter().enumerate() {
        let Some(msg) = line.strip_prefix("FAIL: ") else {
            continue;
        };
        let mut text = msg.to_string();
        let mut found = best(&sites, &text);
        let mut j = k + 1;
        while found.is_err()
            && j < lines.len()
            && !lines[j].starts_with("FAIL: ")
            && !lines[j].starts_with("PASS: ")
        {
            text = format!("{text}\n{}", lines[j]);
            found = best(&sites, &text);
            j += 1;
        }
        match found {
            Ok(c) => {
                out.insert(c);
            }
            Err(why) => unread.push(format!("{msg:?}: {why}")),
        }
    }
    if unread.is_empty() {
        Ok(out)
    } else {
        Err(unread)
    }
}

fn best(sites: &[Site], msg: &str) -> Result<String, String> {
    let hits: Vec<&Site> = sites.iter().filter(|s| s.regex.is_match(msg)).collect();
    let Some(top) = hits.iter().map(|s| s.literal).max() else {
        return Err("no `bad` call site of the script prints it".into());
    };
    let labels: BTreeSet<&str> = hits
        .iter()
        .filter(|s| s.literal == top)
        .map(|s| s.criterion.as_str())
        .collect();
    if labels.len() == 1 {
        Ok(labels.into_iter().next().unwrap_or_default().to_string())
    } else {
        let lines: Vec<String> = hits
            .iter()
            .filter(|s| s.literal == top)
            .map(|s| s.line.to_string())
            .collect();
        Err(format!(
            "call sites of different criteria print it (script lines {})",
            lines.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = r#"#!/usr/bin/env bash
ok()  { printf 'PASS: %s\n' "$*"; pass=$((pass + 1)); }
bad() { printf 'FAIL: %s\n' "$*"; fail=$((fail + 1)); }
# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
[ -f "$R" ] || { bad "report missing: $REPORT_REL"; exit 1; }
# ---------------------------------------------------------------------------
# Locate section (d) and the sub-section inside it
# ---------------------------------------------------------------------------
  bad "section (d) Reading notes not found (heading containing '(d)')"
# ---------------------------------------------------------------------------
# AC 1 — size and pointers
# ---------------------------------------------------------------------------
  bad "sub-section is too long ($nonempty non-empty lines, limit 45 — about one page)"
    bad "exclusion entry lacks$missing: $(printf '%s' "$ln" | cut -c1-110)"
# ---------------------------------------------------------------------------
# Work step 2 — the run
# ---------------------------------------------------------------------------
  [ "$x" -eq 0 ] && bad "AC 1: $label line $k: $o"
  bad 'fixed text' "and more"
# ---------------------------------------------------------------------------
# DoD — tags
# ---------------------------------------------------------------------------
      bad "$label line $k carries a file:line pointer but no evidence tag"
"#;

    #[test]
    fn only_the_selecting_ports_use_the_reader() {
        assert!(selected("002-02") && selected("002-04"));
        assert!(!selected("002-13") && !selected("004-27") && !selected("002-01"));
    }

    #[test]
    fn banners_name_the_criteria() {
        assert_eq!(banner_criterion("AC 1 — size and pointers"), "AC1");
        assert_eq!(banner_criterion("AC-12 — x"), "AC12");
        assert_eq!(banner_criterion("DoD — tags"), "DoD");
        assert_eq!(banner_criterion("Work step 2 — the run"), "Work step 2");
        assert_eq!(
            banner_criterion("Section marker — exactly one"),
            "Section marker"
        );
        assert_eq!(
            banner_criterion("Locate section (d) and its (e) boundary"),
            "Locate"
        );
        assert_eq!(banner_criterion("Helpers"), "Helpers");
    }

    #[test]
    fn messages_map_to_their_call_sites() {
        let s = sites(SCRIPT);
        assert_eq!(s.len(), 7, "{s:?}");
        let out = "PASS: x\nFAIL: report missing: doc/r.md\nFAIL: sub-section is too long (52 non-empty lines, limit 45 — about one page)\nFAIL: exclusion entry lacks pointer reason: - a \"quoted\" (x) line\nFAIL: fixed text and more\nFAIL: (d) line 7 carries a file:line pointer but no evidence tag\n\n3 passed, 4 failed\n";
        let lines: Vec<&str> = out.lines().collect();
        let got = unlabelled_failures(SCRIPT, &lines).expect("all read");
        let want: BTreeSet<String> = ["Helpers", "AC1", "Work step 2", "DoD"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(got, want);
    }

    #[test]
    fn an_unread_message_is_reported_never_dropped() {
        let lines = ["FAIL: something no site prints"];
        assert!(unlabelled_failures(SCRIPT, &lines).is_err());
        // the most literal template wins: "(d) line 7: x" is read by the
        // Work step 2 site (" line ", ": "), and the DoD site is longer only
        // for its own message
        let lines = ["FAIL: AC 1: (d) line 7: x"];
        let got = unlabelled_failures(SCRIPT, &lines).expect("read");
        assert!(got.contains("Work step 2"));
        let multi = ["FAIL: report missing: a", "b"];
        assert!(unlabelled_failures(SCRIPT, &multi).is_ok());
    }
}
