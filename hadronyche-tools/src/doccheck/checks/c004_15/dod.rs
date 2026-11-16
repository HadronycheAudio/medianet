//! DoD: evidence tags on prose, owned open items, and the evidence of the
//! Ethernet framing units (C-132).

use super::num::{Fails, R, ints, sentences};
use super::units::{Sub, Unit};
use crate::doccheck::evidence::{TagSet, WordCount, count_words};
use crate::doccheck::text::{has, has_i, rx};

const FRAMING_TERM: &str = r"(?i)\bFCS\b|frame check|check sequence|preamble|start-of-frame|inter-?frame|\bIFG\b|Ethernet (?:header|framing)";
const CHECKER: &str = r"\b\d{3}[a-z]?-\d{2}\b";
const CHECK_WORD: &str = r"(?i)\b(checks?|checker|pins?|pinned|verifies|verify|checked by)\b";
const CLAUSE: &str = r"(?i)\b(?:sub)?clause\s+\d+(?:\.\d+)+|§\s*\d+(?:\.\d+)+|\b(?:IEEE\s+)?(?:Std\s+)?\d+\.\d+(?:-\d{4})?\s+\d+(?:\.\d+)+";

fn ipv_named(t: &str) -> bool {
    has(r"\bIPv4\b|\bUDP\b", t)
}

/// `is_framing`: names an Ethernet framing part, or "Ethernet" with one of
/// the framing figures; a row labelled IPv4 or UDP is not.
fn is_framing(text: &str, label: Option<&str>) -> R<bool> {
    if label.is_some_and(ipv_named) {
        return Ok(false);
    }
    let body = rx(CHECKER).replace_all(text, " ").into_owned();
    if has(FRAMING_TERM, &body) {
        return Ok(true);
    }
    if !has_i(r"\bEthernet\b", &body) {
        return Ok(false);
    }
    let n = ints(&body)?;
    Ok([14, 4, 8, 12, 38].iter().any(|v| n.contains(v)))
}

fn framing_unit(u: &Unit, f: &mut Fails) -> R<()> {
    let own = &u.own;
    let label = u.row.then(|| u.cells[0].as_str());
    if !is_framing(own, label)? {
        return Ok(());
    }
    let short: String = own.chars().take(80).collect();
    let tags: Vec<String> = rx(r"(?i)\[(?:memory|standard)\]").find_iter(own).map(|m| m.as_str().to_lowercase()).collect();
    if tags.is_empty() {
        f.add("DoD", format!("an Ethernet framing unit has no [memory] or [standard] tag (C-132): {short}"));
        return Ok(());
    }
    if tags.iter().any(|t| t == "[memory]") && !(has(CHECKER, own) && has(CHECK_WORD, own)) {
        f.add("DoD", format!("an Ethernet framing unit tagged [memory] names no checking task (a ticket id with checks/pins/verifies/checker): {short}"));
    }
    if u.row {
        if own.to_lowercase().contains("[standard]") && !has(CLAUSE, own) {
            f.add("DoD", format!("an Ethernet framing row tagged [standard] has no clause reference: {short}"));
        }
        return Ok(());
    }
    for sent in sentences(own) {
        if sent.to_lowercase().contains("[standard]") && is_framing(&sent, None)? && !ipv_named(&sent) && !has(CLAUSE, &sent) {
            let s: String = sent.chars().take(80).collect();
            f.add("DoD", format!("an Ethernet framing sentence tagged [standard] has no clause reference: {s}"));
        }
    }
    Ok(())
}

/// DoD of the subsection.
pub(crate) fn dod(sub: &Sub, f: &mut Fails) -> R<()> {
    for u in sub.units.iter().filter(|u| !u.row) {
        let short: String = u.text.chars().take(80).collect();
        if count_words(&u.text, WordCount::Alnum) >= 5 && !TagSet::Six.found_in(&u.text, false) {
            f.add("DoD", format!("a prose paragraph of 5+ words has no evidence tag: {short}"));
        }
        if has_i(r"^[\s*_]*open item", &u.text) && (!has_i("owner", &u.text) || !has_i("falsifier", &u.text)) {
            f.add("DoD", format!("an open item names no owner or no falsifier: {short}"));
        }
    }
    for u in &sub.units {
        framing_unit(u, f)?;
    }
    Ok(())
}
