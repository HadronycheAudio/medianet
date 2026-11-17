//! Sentence splitting, as each 004 script wrote it.
//!
//! Every script splits with `re.split` on a look-behind (`(?<=[.;])\s+`),
//! which `text::rx` refuses (lead ruling R19: no look-behind). The splitter
//! here reproduces `re.split`'s scan instead: at each position, left to
//! right, the alternatives of the script's pattern are tried in order and the
//! first that matches cuts the text. Each script's pattern, and what it does
//! with the pieces (strip, drop blanks, lower case, re-join), is one
//! [`SentenceSplit`]:
//!
//! | Policy | Script | Python |
//! |---|---|---|
//! | [`SentenceSplit::PERIOD_SEMI`] | 004-09 `prose_sentences`, 004-15 `sentences_of` (prose units), [`super::family004::sentences`] | `[s.strip() for s in re.split(r"(?<=[.;])\s+", t)]` |
//! | [`SentenceSplit::LINES_004_12`] | 004-12 `sents` | `[s.strip() for s in re.split(r"(?<=[.;])\s+\|\n", t) if s.strip()]` |
//! | [`SentenceSplit::NEWLINES_004_20`] | 004-20 `sentences` | `[s for s in re.split(r"(?<=[.;])\s+\|\n+", t) if s.strip()]` |
//! | [`SentenceSplit::PARAGRAPHS_004_29`] | 004-29 `sentences_of` | `[x for x in re.split(r"(?<=[.;])\s+\|\n\s*\n", t) if x.strip()]` |
//! | [`SentenceSplit::BLOCKS_004_25`] | 004-25, 004-28 `sentences` | `re.split(r"(?<=[.;])\s+\|\n\s*\n\|\n(?=[\|*#-])", t)`, blanks dropped, a piece starting with `[` or a backtick joined to the previous one with a space |
//! | [`SentenceSplit::COLON_004_24`] | 004-24 `SENT` | `[s.strip().lower() for s in re.split(r"(?<=[.;:])\s+", t) if s.strip()]` |
//! | [`SentenceSplit::TERMINAL_004_04`] | 004-04 `split_sentences` | `[s.strip() for s in re.split(r"(?<=[.!?])\s+(?=[A-Z`(\[])", t) if s.strip()]` |
//!
//! The scripts also split inline and keep the raw pieces (no strip, no
//! filter): [`split_raw`] with the pattern's alternatives, e.g.
//! [`RAW_PERIOD_SEMI`] (`re.split(r"(?<=[.;])\s+", t)`: 004-09, 11, 14, 15,
//! 16, 19, 20, 21), [`RAW_COLON`] (004-26), [`RAW_COLON_NEWLINES_004_19`],
//! [`RAW_NEWLINE_004_11`], [`RAW_PIPE_004_21`], [`RAW_TERMINAL_004_03`];
//! [`first_sentence_004_29`] is 004-29's `re.split(r"(?<=\.)\s", rest, 1)[0]`.
//!
//! Proven against the scripts' own definitions and inline splits on
//! `data/family004_corpus.txt` (`doccheck::proofs004`).

use super::pycompat::{PyStr, is_space};

/// One alternative of a split pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// `(?<=[chars])\s+`, optionally followed by `(?=[next])`.
    AfterPunct {
        chars: &'static str,
        next: Option<&'static str>,
    },
    /// `\n`.
    Newline,
    /// `\n+`.
    Newlines,
    /// `\n\s*\n`.
    BlankLine,
    /// `\n(?=[chars])`.
    NewlineBefore(&'static str),
    /// A literal character (`\|`).
    Char(char),
    /// `(?<=[chars])\s`: one whitespace character.
    AfterPunctOne(&'static str),
}

/// `(?<=[.;])\s+`.
pub const RAW_PERIOD_SEMI: &[Cut] = &[PERIOD_SEMI];
/// `(?<=[.;:])\s+`.
pub const RAW_COLON: &[Cut] = &[Cut::AfterPunct {
    chars: ".;:",
    next: None,
}];
/// `(?<=[.;:])\s+|\n+` (004-19).
pub const RAW_COLON_NEWLINES_004_19: &[Cut] = &[
    Cut::AfterPunct {
        chars: ".;:",
        next: None,
    },
    Cut::Newlines,
];
/// `(?<=[.;])\s+|\n` (004-11).
pub const RAW_NEWLINE_004_11: &[Cut] = &[PERIOD_SEMI, Cut::Newline];
/// `(?<=[.;])\s+|\|` (004-21).
pub const RAW_PIPE_004_21: &[Cut] = &[PERIOD_SEMI, Cut::Char('|')];
/// `(?<=[.!?])\s+(?=[A-Z`(\[])` (004-03, 004-04).
pub const RAW_TERMINAL_004_03: &[Cut] = &[Cut::AfterPunct {
    chars: ".!?",
    next: Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ`(["),
}];

/// What happens to the pieces after the split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pieces {
    /// `s.strip()` each piece.
    pub strip: bool,
    /// Drop the pieces that are blank (`if s.strip()`).
    pub drop_blank: bool,
    /// `.lower()` each piece.
    pub lower: bool,
    /// Join a piece that starts (after whitespace) with `[` or a backtick to
    /// the previous one, with a space (004-25's tag continuation).
    pub join_tags: bool,
}

/// A script's sentence split: its pattern's alternatives, in order, and
/// what it does with the pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SentenceSplit {
    pub name: &'static str,
    pub cuts: &'static [Cut],
    pub pieces: Pieces,
}

const PERIOD_SEMI: Cut = Cut::AfterPunct {
    chars: ".;",
    next: None,
};

impl SentenceSplit {
    pub const PERIOD_SEMI: SentenceSplit = SentenceSplit {
        name: "period-semi",
        cuts: &[PERIOD_SEMI],
        pieces: Pieces {
            strip: true,
            drop_blank: false,
            lower: false,
            join_tags: false,
        },
    };
    pub const LINES_004_12: SentenceSplit = SentenceSplit {
        name: "lines-004-12",
        cuts: &[PERIOD_SEMI, Cut::Newline],
        pieces: Pieces {
            strip: true,
            drop_blank: true,
            lower: false,
            join_tags: false,
        },
    };
    pub const NEWLINES_004_20: SentenceSplit = SentenceSplit {
        name: "newlines-004-20",
        cuts: &[PERIOD_SEMI, Cut::Newlines],
        pieces: Pieces {
            strip: false,
            drop_blank: true,
            lower: false,
            join_tags: false,
        },
    };
    pub const PARAGRAPHS_004_29: SentenceSplit = SentenceSplit {
        name: "paragraphs-004-29",
        cuts: &[PERIOD_SEMI, Cut::BlankLine],
        pieces: Pieces {
            strip: false,
            drop_blank: true,
            lower: false,
            join_tags: false,
        },
    };
    pub const BLOCKS_004_25: SentenceSplit = SentenceSplit {
        name: "blocks-004-25",
        cuts: &[PERIOD_SEMI, Cut::BlankLine, Cut::NewlineBefore("|*#-")],
        pieces: Pieces {
            strip: false,
            drop_blank: true,
            lower: false,
            join_tags: true,
        },
    };
    pub const COLON_004_24: SentenceSplit = SentenceSplit {
        name: "colon-004-24",
        cuts: &[Cut::AfterPunct {
            chars: ".;:",
            next: None,
        }],
        pieces: Pieces {
            strip: true,
            drop_blank: true,
            lower: true,
            join_tags: false,
        },
    };
    pub const TERMINAL_004_04: SentenceSplit = SentenceSplit {
        name: "terminal-004-04",
        cuts: &[Cut::AfterPunct {
            chars: ".!?",
            next: Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ`(["),
        }],
        pieces: Pieces {
            strip: true,
            drop_blank: true,
            lower: false,
            join_tags: false,
        },
    };
}

/// The length in bytes of the match of `cut` at byte `p` of `t`, if any.
fn match_at(t: &str, p: usize, cut: Cut) -> Option<usize> {
    let rest = &t[p..];
    let ws_run = |s: &str| {
        s.char_indices()
            .find(|(_, c)| !is_space(*c))
            .map_or(s.len(), |(i, _)| i)
    };
    match cut {
        Cut::AfterPunct { chars, next } => {
            let prev = t[..p].chars().next_back()?;
            if !chars.contains(prev) {
                return None;
            }
            let n = ws_run(rest);
            if n == 0 {
                return None;
            }
            // `\s+(?=[next])`: greedy, and a shorter run ends before
            // whitespace, which is never in `next`
            match next {
                Some(set) if !rest[n..].chars().next().is_some_and(|c| set.contains(c)) => None,
                _ => Some(n),
            }
        }
        Cut::Newline => rest.starts_with('\n').then_some(1),
        Cut::Newlines => {
            let n = rest.len() - rest.trim_start_matches('\n').len();
            (n > 0).then_some(n)
        }
        Cut::BlankLine => {
            // `\n\s*\n`: up to the last `\n` of the whitespace run after
            // the first one (backtracking from the greedy `\s*`)
            let after = rest.strip_prefix('\n')?;
            let run = &after[..ws_run(after)];
            run.rfind('\n').map(|k| 1 + k + 1)
        }
        Cut::NewlineBefore(set) => {
            let after = rest.strip_prefix('\n')?;
            after.chars().next().filter(|c| set.contains(*c)).map(|_| 1)
        }
        Cut::Char(ch) => rest.starts_with(ch).then_some(ch.len_utf8()),
        Cut::AfterPunctOne(chars) => {
            let prev = t[..p].chars().next_back()?;
            let c = rest.chars().next()?;
            (chars.contains(prev) && is_space(c)).then_some(c.len_utf8())
        }
    }
}

/// `re.split(pattern, text)` for the alternatives `cuts`: the raw pieces.
pub fn split_raw<'t>(t: &'t str, cuts: &[Cut]) -> Vec<&'t str> {
    raw_split(t, cuts, usize::MAX)
}

/// 004-29's `re.split(r"(?<=\.)\s", rest, 1)[0]`: the text up to the first
/// whitespace character after a full stop.
pub fn first_sentence_004_29(t: &str) -> &str {
    raw_split(t, &[Cut::AfterPunctOne(".")], 1)[0]
}

/// `re.split(pattern, text, maxsplit)` for the alternatives `cuts`.
fn raw_split<'t>(t: &'t str, cuts: &[Cut], maxsplit: usize) -> Vec<&'t str> {
    let mut out = Vec::new();
    let (mut last, mut p) = (0, 0);
    while p <= t.len() && out.len() < maxsplit {
        if let Some(n) = cuts.iter().find_map(|c| match_at(t, p, *c)) {
            out.push(&t[last..p]);
            p += n;
            last = p;
            continue;
        }
        match t[p..].chars().next() {
            Some(c) => p += c.len_utf8(),
            None => break,
        }
    }
    out.push(&t[last..]);
    out
}

/// The sentences of `text` under `how`.
pub fn split(text: &str, how: SentenceSplit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for piece in split_raw(text, how.cuts) {
        if how.pieces.drop_blank && piece.py_strip().is_empty() {
            continue;
        }
        let mut s = if how.pieces.strip {
            piece.py_strip().to_string()
        } else {
            piece.to_string()
        };
        if how.pieces.lower {
            s = s.to_lowercase();
        }
        if how.pieces.join_tags
            && let Some(prev) = out.last_mut()
            && s.py_lstrip().starts_with(['[', '`'])
        {
            prev.push(' ');
            prev.push_str(&s);
            continue;
        }
        out.push(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_follow_re_split() {
        let t = "A b. C; d\ne.  F\n\n\n[code] g";
        assert_eq!(
            split(t, SentenceSplit::PERIOD_SEMI),
            ["A b.", "C;", "d\ne.", "F\n\n\n[code] g"]
        );
        assert_eq!(
            split(t, SentenceSplit::LINES_004_12),
            ["A b.", "C;", "d", "e.", "F", "[code] g"]
        );
        assert_eq!(
            split(t, SentenceSplit::PARAGRAPHS_004_29),
            ["A b.", "C;", "d\ne.", "F", "[code] g"]
        );
        assert_eq!(
            split(t, SentenceSplit::BLOCKS_004_25),
            ["A b.", "C;", "d\ne.", "F [code] g"]
        );
        assert_eq!(
            split("x: Y. z", SentenceSplit::COLON_004_24),
            ["x:", "y.", "z"]
        );
        assert_eq!(
            split("a. b. C! (d", SentenceSplit::TERMINAL_004_04),
            ["a. b.", "C!", "(d"]
        );
        assert_eq!(split("a\n \n b", SentenceSplit::BLOCKS_004_25), ["a", " b"]);
        assert_eq!(split_raw("a. b|c", RAW_PIPE_004_21), ["a.", "b", "c"]);
        assert_eq!(
            first_sentence_004_29("Open item 3.  Owner: x. y"),
            "Open item 3."
        );
    }
}
