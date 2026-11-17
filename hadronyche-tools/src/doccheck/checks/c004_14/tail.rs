//! DoD (evidence tags and open items in the section) and the pointers and
//! landed sentences outside it.

use super::section::Sec;
use super::section::norm;
use crate::doccheck::evidence::{TagSet, WordCount, count_words};
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::report::Report;
use crate::doccheck::text::{has, has_i, rx};

/// The paragraph units of the section: a blank line, a heading, a table row
/// or a list marker (`-`, `*`, `+`, `1.`) ends a unit; fenced lines are
/// skipped without ending one.
fn units(sec: &Sec) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for (line, fence) in sec.sec_lines.iter().zip(&sec.sec_fence) {
        let s = line.py_strip();
        if *fence {
            continue;
        }
        if s.is_empty() || s.starts_with('#') || line.py_lstrip().starts_with('|') {
            if !cur.is_empty() {
                out.push(cur.join(" "));
                cur.clear();
            }
            continue;
        }
        if has(r"^([-*+]\s|\d+\.\s)", s) && !cur.is_empty() {
            out.push(cur.join(" "));
            cur.clear();
        }
        cur.push(s.to_string());
    }
    if !cur.is_empty() {
        out.push(cur.join(" "));
    }
    out
}

/// DoD: every unit of 5 or more `[A-Za-z0-9]+` words carries a tag; an open
/// item names an owner and a falsifier.
pub(crate) fn dod(sec: &Sec, f: &mut Report) {
    for u in units(sec) {
        let short: String = u.chars().take(80).collect();
        if count_words(&u, WordCount::Alnum) >= 5 && !TagSet::Six.found_in(&u, false) {
            f.fail(
                "DoD",
                format!("a prose paragraph of 5+ words has no evidence tag: {short}"),
            );
        }
        let stripped = rx(r"^[\s*_]+").replace(&u, "");
        if has_i(r"^open item", &stripped) && (!has_i("owner", &u) || !has_i("falsifier", &u)) {
            f.fail(
                "DoD",
                format!("an open item in the section names no owner or no falsifier: {short}"),
            );
        }
    }
}

const LANDED: [&str; 5] = [
    "All four announcements fit inside the 100-byte cap",
    "the container never blocks that cadence",
    "the extension fits inside the cap in every v1 packet",
    "The header cap holds with no fallback rule",
    "no v1 sender omits it on a fallback path",
];

/// The pointers and landed sentences of wire.md outside the section.
pub(crate) fn pointers(wire: &str, sec: &Sec, f: &mut Report) {
    let wire_norm = norm(wire);
    let glob_norm = norm(&sec.glob_lines.join(" "));
    if wire_norm.contains("open item owned by 004-14") {
        f.fail("pointer", "the string 'open item owned by 004-14' remains in wire.md; the Size budget section must close it");
    }
    if has(r"But see[^)]{0,200}?\bBudget\b", &wire_norm) {
        f.fail("pointer", "a 'But see ... Budget' pointer remains in wire.md; resolve it or cite the Size budget section");
    }
    if has_i(
        r"(?:open item|overflow)[^.]{0,120}?004-14|004-14[^.]{0,40}?\(Budget\)",
        &glob_norm,
    ) {
        f.fail(
            "pointer",
            "an unresolved pointer to 004-14 (Budget) remains outside the Size budget section",
        );
    }
    provisional(sec, f);
    for phrase in LANDED {
        let want = phrase.to_lowercase();
        if sec
            .glob_lines
            .iter()
            .any(|l| norm(l).to_lowercase().contains(&want) && !has_i("size budget", l))
        {
            f.fail("pointer", format!("the landed sentence {phrase:?} is neither removed nor cited to the Size budget section"));
        }
    }
    for line in &sec.glob_lines {
        let stripped = rx(r"^[\s*_]+").replace(line.py_strip(), "");
        if has_i(r"^Open item \(header cap\)", &stripped)
            && !has_i(r"\bclosed\b|\bresolved\b|\banswered\b", line)
        {
            f.fail("pointer", "the header-cap open item of Content messages, Budget is not marked closed or resolved");
        }
    }
}

fn provisional(sec: &Sec, f: &mut Report) {
    let stays = has_i("stays provisional", &sec.sec_text) && has_i(r"\bbecause\b", &sec.sec_text);
    for line in &sec.glob_lines {
        if line.to_lowercase().contains("provisional")
            && has_i(r"004-14|shared-packet|\bcap\b", line)
            && !stays
        {
            let short: String = norm(line).chars().take(90).collect();
            f.fail(
                "pointer",
                format!("a 'provisional' cap or shared-packet line remains: {short}"),
            );
            break;
        }
    }
}
