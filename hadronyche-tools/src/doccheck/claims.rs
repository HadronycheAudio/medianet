//! Positive statements, verdict words and designations, for NEW checks.
//!
//! Scope (lead ruling R15): natural-language negation cannot be made
//! complete. These are keyword detectors, not semantics. A PORT never uses
//! them in place of its script's own regex: it reproduces the script's
//! pattern exactly. A new check may use them, knowing what they handle:
//!
//! * negation words: not, never, no, nor, cannot, without, neither, any
//!   `n't`, nobody, none (also "none of"), nothing, nowhere, hardly,
//!   scarcely, barely, "far from", "failed to"/"fails to"/"fail to";
//! * not negations: "not only", "without doubt", "no doubt";
//! * a clause ends at `.` `;` `:` `!` `?` (not inside a number), a comma, a
//!   dash, a parenthesis, "but", "whereas", "while"; a sentence that ends
//!   with `?` states nothing;
//! * [`must_affirm`] ("the document must say X"): X with no negation word
//!   before it in its clause;
//! * [`claims`] ("the document must not say X"): X counts as said unless an
//!   odd number of negation words stand before it AFTER the last of "and",
//!   "or", "that", "which", "because", "if", "so" in its clause ("Not
//!   refuted and confirmed" says "confirmed"; "never was it doubted that it
//!   is confirmed" says it).
//!
//! Known limits: a negation in a governing clause before "that" is not seen
//! by [`claims`] ("it is not true that it was never confirmed" does not
//! count as saying "confirmed"); negative verbs and adjectives ("denied",
//! "lacks", "unconfirmed", "doubtful") are not negations; irony, scope over
//! lists, and negation carried across sentences are not handled; quoted
//! speech counts as stated.
//!
//! A verdict is exactly one of a set of words stated positively; a
//! designation is a pattern that no bare year or year range satisfies.

use super::text::rx;

/// The negation words of a clause.
pub const NEGATION: &str = concat!(
    r"(?i)\b(?:not|never|no|nor|cannot|without|neither|nobody|none|nothing|nowhere|hardly|scarcely|barely)\b",
    r"|n't\b|n’t\b|(?i:\bfar\s+from\b|\bfail(?:s|ed)?\s+to\b)"
);

/// Phrases that contain a negation word but do not negate.
const NOT_NEGATION: &str = r"(?i)^(?:not\s+only\b|without\s+(?:a\s+)?doubt\b|no\s+doubt\b)";

/// Words after which a negation before them no longer governs ([`claims`]).
const SCOPE_BREAK: &str = r"(?i)\b(?:and|or|that|which|because|if|so)\b";

/// The clauses of `text` that are statements: cut at sentence punctuation
/// (`.` `;` `:` `!` `?` not inside a number), commas, dashes, parentheses and
/// the conjunctions "but", "whereas", "while"; the clauses of a sentence that
/// ends with `?` are dropped.
pub fn clauses(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    for m in rx(r"[.!?](?:\s|$)").find_iter(text) {
        let sentence = &text[start..m.end()];
        start = m.end();
        if !sentence.trim_end().ends_with('?') {
            out.extend(sentence_clauses(sentence));
        }
    }
    out.extend(sentence_clauses(&text[start..]));
    out
}

fn sentence_clauses(text: &str) -> Vec<&str> {
    let cut = rx(r"(?i)[;:!?,()—–]|\.(?:\s|$)|\s-\s|\b(?:but|whereas|while)\b");
    let mut out = Vec::new();
    let mut start = 0;
    for m in cut.find_iter(text) {
        out.push(&text[start..m.start()]);
        start = m.end();
    }
    out.push(&text[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect()
}

/// True when `clause` holds a negation word anywhere.
pub fn is_negated(clause: &str) -> bool {
    negations_in(clause, 0, clause.len()) > 0
}

/// The negation words of `clause[from..to]`, the non-negating phrases left
/// out.
fn negations_in(clause: &str, from: usize, to: usize) -> usize {
    rx(NEGATION)
        .find_iter(&clause[..to])
        .filter(|m| m.start() >= from && !rx(NOT_NEGATION).is_match(&clause[m.start()..]))
        .count()
}

/// The negation words of `clause` before byte `at` ([`must_affirm`]).
pub fn negations_before(clause: &str, at: usize) -> usize {
    negations_in(clause, 0, at)
}

/// The negation words before byte `at` that still govern it: those after
/// the last scope break ([`claims`]).
fn governing_negations(clause: &str, at: usize) -> usize {
    let from = rx(SCOPE_BREAK)
        .find_iter(&clause[..at])
        .last()
        .map_or(0, |m| m.end());
    negations_in(clause, from, at)
}

/// Direction "must affirm": true when some clause of `text` states
/// `pattern` with no negation word before it in the clause (see the module
/// documentation for the forms handled and the limits). Never for a
/// prohibition: use [`claims`].
pub fn must_affirm(text: &str, pattern: &str) -> bool {
    let re = rx(pattern);
    clauses(text)
        .iter()
        .any(|c| re.find_iter(c).any(|m| negations_before(c, m.start()) == 0))
}

/// Direction "must not claim": true when some clause of `text` states
/// `pattern` without an odd number of governing negation words before it
/// (see the module documentation).
pub fn claims(text: &str, pattern: &str) -> bool {
    let re = rx(pattern);
    clauses(text).iter().any(|c| {
        re.find_iter(c)
            .any(|m| governing_negations(c, m.start()).is_multiple_of(2))
    })
}

/// The clauses of `text` that state `pattern` positively ([`must_affirm`]).
pub fn positive_clauses<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    let re = rx(pattern);
    clauses(text)
        .into_iter()
        .filter(|c| re.find_iter(c).any(|m| negations_before(c, m.start()) == 0))
        .collect()
}

/// [`must_affirm`], under its former name.
pub fn affirms(text: &str, pattern: &str) -> bool {
    must_affirm(text, pattern)
}

/// The verdict words of `words` that `text` states positively (each matched
/// as a whole word, any case), in the order of `words`.
pub fn affirmed_words<'w>(text: &str, words: &[&'w str]) -> Vec<&'w str> {
    words
        .iter()
        .copied()
        .filter(|w| affirms(text, &format!(r"(?i)\b{}\b", regex::escape(w))))
        .collect()
}

/// `Ok(word)` when `text` states exactly one of `words` positively;
/// otherwise `Err` with the words it states (none, or several).
pub fn exactly_one_of<'w>(text: &str, words: &[&'w str]) -> Result<&'w str, Vec<&'w str>> {
    let found = affirmed_words(text, words);
    if found.len() == 1 {
        Ok(found[0])
    } else {
        Err(found)
    }
}

/// The kinds of document designation a check may require.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Designation {
    /// "2nd edition", "edition 3", "Ed. 2", "third edition" (an edition
    /// number has one or two digits); or a document identifier followed by
    /// its year: optional letters ("IEEE ", "ST ", "G.", "AES", "ISO/IEC ")
    /// and a number whose leading digits are two or more and are not
    /// themselves a year ("802.1AS-2020", "G.8275.1-2016", "ST 2110-30:2017",
    /// "IEEE 1588-2019", "AES67-2018", "IEC 61883-6:2005"). No bare year or
    /// year range is an edition, whatever precedes it ("2019-2020", "IEEE
    /// 2019-2020", "FY2019-2020", "Q3-2019", "May 1-2019").
    Edition,
    /// "draft-ietf-tsvwg-foo-05", "Draft 3", "D3.1" after a document number.
    Draft,
    /// "comment record" followed by an identifier with a digit.
    CommentRecord,
    /// "test suite" followed by a version number.
    TestSuite,
    /// A working-group document dated with a month and a year.
    WorkingGroupDocument,
}

const MONTHS: &str = "January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec";

/// The regular expression of a designation kind.
pub fn designation_pattern(kind: Designation) -> String {
    match kind {
        Designation::Edition => concat!(
            r"(?i:\b(?:\d+(?:st|nd|rd|th)|first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth)\s+edition\b)",
            r"|(?i:\bedition\s+\d{1,2}(?:\.\d+)?\b)|\bEd\.\s?\d{1,2}\b"
        )
        .to_string(),
        Designation::Draft => {
            r"(?i)\bdraft-[a-z0-9]+(?:-[a-z0-9]+)*-\d{2}\b|\bdraft\s+\d+(?:\.\d+)?\b|/D\d+(?:\.\d+)?\b".to_string()
        }
        Designation::CommentRecord => {
            r"(?i)\bcomment\s+record\s+(?:no\.?\s*|number\s+|#)?[A-Za-z-]*\d[\w.-]*".to_string()
        }
        Designation::TestSuite => {
            r"(?i)\btest\s+suite\s+(?:version\s+|v\.?\s?|release\s+)?\d+(?:\.\d+)*\b".to_string()
        }
        Designation::WorkingGroupDocument => format!(
            r"(?i)\b(?:working[- ]group|WG)\b[^.]{{0,120}}?\b(?:{MONTHS})\.?\s+(?:19|20)\d\d\b"
        ),
    }
}

/// A document identifier and its year (see [`Designation::Edition`]): the
/// candidates, filtered so that the identifier's number is not a year.
const ID_YEAR: &str = r"\b((?:[A-Z][A-Za-z]*(?:/[A-Z]+)?[.\s]?)*)(\d+(?:\.\d+)*[A-Za-z]*(?:-\d+)*)[-:]((?:19|20)\d\d)\b";

fn identifier_year(text: &str) -> Vec<String> {
    rx(ID_YEAR)
        .captures_iter(text)
        .filter(|c| {
            let lead: String = c[2].chars().take_while(char::is_ascii_digit).collect();
            let year_like = lead.len() == 4 && (lead.starts_with("19") || lead.starts_with("20"));
            lead.len() >= 2 && !year_like
        })
        .map(|c| c[0].to_string())
        .collect()
}

/// The designations of `kind` in `text`, as written.
pub fn designations(text: &str, kind: Designation) -> Vec<String> {
    let mut out: Vec<String> = rx(&designation_pattern(kind))
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect();
    if kind == Designation::Edition {
        out.extend(identifier_year(text));
    }
    out
}

/// True when `text` holds a designation of `kind` in a clause with no
/// negation word ([`must_affirm`]'s direction).
pub fn states_designation(text: &str, kind: Designation) -> bool {
    clauses(text)
        .iter()
        .any(|c| !is_negated(c) && !designations(c, kind).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_denial_is_not_a_statement() {
        assert!(affirms("The result is confirmed.", "(?i)confirmed"));
        assert!(!affirms("The result is not confirmed.", "(?i)confirmed"));
        assert!(!affirms("It was never confirmed", "(?i)confirmed"));
        assert!(!affirms("It isn't confirmed", "(?i)confirmed"));
        assert!(!affirms("Neither confirmed nor refuted", "(?i)confirmed"));
        assert!(!affirms("Closed without being confirmed", "(?i)confirmed"));
        assert!(!affirms("We cannot say it is confirmed", "(?i)confirmed"));
        assert!(affirms("Not refuted; confirmed by run 3", "(?i)confirmed"));
        assert!(affirms("Not refuted, but confirmed", "(?i)confirmed"));
    }

    #[test]
    fn the_two_directions() {
        // "confirmed and not refuted"
        let t = "The result is confirmed and not refuted.";
        assert!(must_affirm(t, "(?i)confirmed"));
        assert!(!must_affirm(t, "(?i)refuted"));
        assert!(!claims(t, "(?i)refuted"), "a single negation before it");
        assert!(claims(t, "(?i)confirmed"));
        // "not only ... but also"
        let t = "It is not only confirmed but also reproduced.";
        assert!(must_affirm(t, "(?i)confirmed"));
        assert!(must_affirm(t, "(?i)reproduced"));
        // double negation
        let t = "It is not true that it was never confirmed.";
        assert!(
            !must_affirm(t, "(?i)confirmed"),
            "must affirm wants a plain statement"
        );
        assert!(
            !claims(t, "(?i)confirmed"),
            "known limit: the negation before 'that' is not seen"
        );
        // a guard word in another sentence
        let t = "It was not tested. It is confirmed.";
        assert!(must_affirm(t, "(?i)confirmed"));
        assert!(claims(t, "(?i)confirmed"));
        let t = "It was not confirmed.";
        assert!(!must_affirm(t, "(?i)confirmed") && !claims(t, "(?i)confirmed"));
    }

    #[test]
    fn the_review_round_2_sentences() {
        let c = "(?i)confirmed";
        for t in [
            "Not refuted and confirmed.",
            "Never was it doubted that it is confirmed.",
            "Without doubt confirmed.",
        ] {
            assert!(claims(t, c), "claims: {t}");
        }
        for t in [
            "Nobody confirmed it.",
            "None of the runs confirmed it.",
            "It is hardly confirmed.",
            "It is far from confirmed.",
            "It failed to be confirmed.",
            "It was confirmed? No.",
        ] {
            assert!(!must_affirm(t, c), "must_affirm: {t}");
        }
        assert!(must_affirm("No doubt it is confirmed.", c));
    }

    #[test]
    fn clauses_keep_decimal_numbers_whole() {
        assert_eq!(
            clauses("Width 1.5 bits. Next"),
            vec!["Width 1.5 bits", "Next"]
        );
        assert_eq!(clauses("a; b: c (d) e"), vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn exactly_one_verdict_word() {
        let words = ["confirmed", "refuted", "inconclusive"];
        assert_eq!(
            exactly_one_of("Verdict: confirmed.", &words),
            Ok("confirmed")
        );
        assert_eq!(
            exactly_one_of("Not confirmed; refuted.", &words),
            Ok("refuted")
        );
        assert_eq!(exactly_one_of("Not confirmed.", &words), Err(vec![]));
        assert_eq!(
            exactly_one_of("confirmed. refuted.", &words),
            Err(vec!["confirmed", "refuted"])
        );
    }

    #[test]
    fn a_bare_year_is_not_an_edition() {
        let ed = Designation::Edition;
        assert!(designations("published in 2019", ed).is_empty());
        assert!(designations("the 2019 text", ed).is_empty());
        assert!(designations("edition 2019", ed).is_empty());
        for bare in [
            "In 2019-2020",
            "Since 2018-2019",
            "From 2019:2020",
            "Q3-2019",
            "May 1-2019",
            "2019-2020",
        ] {
            assert!(designations(bare, ed).is_empty(), "{bare}");
        }
        assert!(!designations("ISO/IEC 9899:2018", ed).is_empty());
        for bare in [
            "IEEE 2019-2020",
            "ISO 2019:2020",
            "RFC 2020-2021",
            "AES 2019-2020",
            "FY2019-2020",
            "CY2020-2021",
        ] {
            assert!(designations(bare, ed).is_empty(), "{bare}");
        }
        for real in [
            "802.1AS-2020",
            "ITU-T G.8275.1-2016",
            "SMPTE ST 2110-30:2017",
            "IEEE 802.1AS-2020",
            "AES67-2018",
        ] {
            assert!(!designations(real, ed).is_empty(), "{real}");
        }
        assert_eq!(
            designations("IEEE 1588-2019 clause 7", ed),
            vec!["IEEE 1588-2019"]
        );
        assert_eq!(designations("AES67-2018", ed), vec!["AES67-2018"]);
        assert_eq!(
            designations("IEC 61883-6:2005", ed),
            vec!["IEC 61883-6:2005"]
        );
        assert!(!designations("the 2nd edition", ed).is_empty());
        assert!(!designations("edition 3", ed).is_empty());
        assert!(!designations("Ed. 2", ed).is_empty());
        assert!(!states_designation("not the 2nd edition", ed));
    }

    #[test]
    fn other_designations() {
        assert!(!designations("draft-ietf-tsvwg-l4s-05", Designation::Draft).is_empty());
        assert!(!designations("P802.1AS/D3.0", Designation::Draft).is_empty());
        assert!(designations("a draft of 2020", Designation::Draft).is_empty());
        assert!(!designations("comment record CR-12", Designation::CommentRecord).is_empty());
        assert!(designations("the comment record", Designation::CommentRecord).is_empty());
        assert!(!designations("test suite version 2.1", Designation::TestSuite).is_empty());
        assert!(designations("the test suite", Designation::TestSuite).is_empty());
        let wg = Designation::WorkingGroupDocument;
        assert!(!designations("the working group's minutes of March 2024", wg).is_empty());
        assert!(designations("the working group in 2024", wg).is_empty());
    }
}
