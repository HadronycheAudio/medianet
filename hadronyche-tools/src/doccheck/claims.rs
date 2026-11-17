//! Positive statements, verdict words and designations.
//!
//! The recurrent defect this module exists for: a keyword regex satisfied by
//! its own denial ("not confirmed" matching "confirmed"). A positive
//! statement is a keyword match in a clause with no negation word before it,
//! in one of two explicit directions ([`must_affirm`], [`claims`]); a verdict
//! is exactly one of a set of words stated positively; a designation is a
//! pattern that a bare year never satisfies.

use super::text::rx;

/// The negation words of a clause: not, never, no, nor, cannot, without,
/// neither, and any `n't` contraction.
pub const NEGATION: &str = r"(?i)\b(?:not|never|no|nor|cannot|without|neither)\b|n't\b|n’t\b";

/// The clauses of `text`: cut at sentence punctuation (`.` `;` `:` `!` `?`
/// not inside a number), commas, dashes, parentheses and the conjunctions
/// "but", "whereas", "while".
pub fn clauses(text: &str) -> Vec<&str> {
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
    rx(NEGATION).is_match(clause)
}

/// The negation words of `clause` before byte `at`, "not only" excluded
/// (it introduces an addition, not a denial).
pub fn negations_before(clause: &str, at: usize) -> usize {
    let head = &clause[..at];
    rx(NEGATION)
        .find_iter(head)
        .filter(|m| !rx(r"(?i)^not\s+only\b").is_match(&clause[m.start()..]))
        .count()
}

fn matches_with_negations<'a>(text: &'a str, pattern: &str) -> Vec<(&'a str, usize)> {
    let re = rx(pattern);
    let mut out = Vec::new();
    for c in clauses(text) {
        for m in re.find_iter(c) {
            out.push((c, negations_before(c, m.start())));
        }
    }
    out
}

/// Direction "must affirm": true when some clause of `text` states
/// `pattern` with no negation word before it in the clause. "Confirmed and
/// not refuted" affirms "confirmed"; "not confirmed" and a double negation
/// ("not true that it was never confirmed") do not; a negation in another
/// sentence or clause does not count. Use it where the document must SAY
/// something; never for a prohibition (see [`claims`]).
pub fn must_affirm(text: &str, pattern: &str) -> bool {
    matches_with_negations(text, pattern)
        .iter()
        .any(|(_, n)| *n == 0)
}

/// Direction "must not claim": true when some clause of `text` states
/// `pattern` with an even number of negation words before it (none, or a
/// double negation, which affirms). Only a single negation before the
/// keyword in the same clause makes a mention harmless. Use it where the
/// document must NOT say something.
pub fn claims(text: &str, pattern: &str) -> bool {
    matches_with_negations(text, pattern)
        .iter()
        .any(|(_, n)| n % 2 == 0)
}

/// The clauses of `text` that state `pattern` positively ([`must_affirm`]).
pub fn positive_clauses<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    let mut out: Vec<&str> = matches_with_negations(text, pattern)
        .into_iter()
        .filter(|(_, n)| *n == 0)
        .map(|(c, _)| c)
        .collect();
    out.dedup();
    out
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
    /// number has one or two digits, so "edition 2019" is not one); or a document
    /// number with its year: a standards body prefix (IEEE, IEC, ISO, AES,
    /// ITU-T, ANSI, SMPTE, EBU, ETSI, RFC) and a number, or letters joined to
    /// a number of two digits or more ("IEEE 1588-2019", "AES67-2018",
    /// "IEC 61883-6:2005"). A bare year or year range ("2019-2020", "In
    /// 2019-2020", "Q3-2019", "May 1-2019") is not an edition.
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
            r"|(?i:\bedition\s+\d{1,2}(?:\.\d+)?\b)|\bEd\.\s?\d{1,2}\b",
            r"|\b(?:(?:IEEE|IEC|ISO|ISO/IEC|AES|ITU-T|ANSI|SMPTE|EBU|ETSI|RFC)\s?[A-Z]?\d[\d.]*(?:-\d+)*|[A-Z]{2,}\d{2,}(?:-\d+)*)[-:](?:19|20)\d\d\b"
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

/// The designations of `kind` in `text`, as written.
pub fn designations(text: &str, kind: Designation) -> Vec<String> {
    rx(&designation_pattern(kind))
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// True when `text` holds a designation of `kind` in a positive clause.
pub fn states_designation(text: &str, kind: Designation) -> bool {
    affirms(text, &designation_pattern(kind))
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
        assert!(claims(t, "(?i)confirmed"), "a double negation is a claim");
        // a guard word in another sentence
        let t = "It was not tested. It is confirmed.";
        assert!(must_affirm(t, "(?i)confirmed"));
        assert!(claims(t, "(?i)confirmed"));
        let t = "It was not confirmed.";
        assert!(!must_affirm(t, "(?i)confirmed") && !claims(t, "(?i)confirmed"));
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
