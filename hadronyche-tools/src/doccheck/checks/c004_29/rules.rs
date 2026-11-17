//! AC3: the receiver rules — the open, expiry and explicit-close
//! transitions as three different numbered rules (review T1), the second
//! announcement, two modules with different windows (review F2), and the
//! unusable time with a lost close (review F7).

use super::Section;
use crate::doccheck::Report;
use crate::doccheck::evidence::has_tag;
use crate::doccheck::items::{OpenItemPolicy, open_items};
use crate::doccheck::pycompat::{PyStr, splitlines};
use crate::doccheck::sentences::first_sentence_004_29;
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// `parse_rules`: each numbered (`1. `) or `**Rule N` item with its
/// continuation lines, up to a blank line.
pub(super) fn parse_rules(text: &str) -> Vec<String> {
    let mut rules: Vec<Vec<&str>> = Vec::new();
    let mut open = false;
    for line in splitlines(text) {
        if rx(r"^\s*(\d+\.\s|\*\*Rule \d+)").is_match(line) {
            rules.push(vec![line]);
            open = true;
        } else if line.py_strip().is_empty() {
            open = false;
        } else if open {
            rules.last_mut().expect("a rule is open").push(line);
        }
    }
    rules.into_iter().map(|r| r.join(" ")).collect()
}

/// `rule_parts`: (bold label, lowered first sentence of the rest).
pub(super) fn rule_parts(rule: &str) -> (String, String) {
    let label = rx(r"\*\*([^*]+)\*\*").captures(rule).map(|c| c[1].py_strip().trim_end_matches('.').to_lowercase()).unwrap_or_default();
    let rest = rx(r"^\s*(\d+\.|\*\*Rule \d+\*\*)\s*").replace(rule, "");
    let rest = rx(r"\*\*[^*]+\*\*").replace(&rest, "");
    (label, first_sentence_004_29(rest.py_strip()).to_lowercase())
}

const CLOSE_VERB: &str = r"closes (?:its |the |it\b)|window closes|closes it\b|ends the window";
const NEG_STATE: &str = r"stays|remember|memor|already|past|\bafter\b|closed window|ignores";

fn transition_cands(info: &[(String, String)], label_pos: &str, label_neg: &str, first_pos: &str, close_verb: bool) -> Vec<usize> {
    info.iter()
        .enumerate()
        .filter(|(_, (lab, first))| {
            let text = format!("{lab} {first}");
            has(label_pos, lab) && !has(label_neg, lab) && !has(NEG_STATE, &text) && has(first_pos, first) && (!close_verb || has(CLOSE_VERB, first))
        })
        .map(|(i, _)| i)
        .collect()
}

fn transitions(s: &Section, report: &mut Report) {
    let info: Vec<(String, String)> = parse_rules(&s.rul).iter().map(|r| rule_parts(r)).collect();
    let open = transition_cands(&info, r"\bopen", "^$", r"\bopens\b", false);
    let exp = transition_cands(&info, "expir", "unknown|late|copy|repeat|after|second", "expir|reached", true);
    let expl = transition_cands(&info, "explicit|close", "unknown|late|copy|repeat|after|already|second", "explicit close|close message|a close|valid close", true);
    report.check("AC3", !open.is_empty(), format!("the receiver rules hold a numbered rule for the open transition (found rules {open:?})"));
    report.check("AC3", !exp.is_empty(), format!("the receiver rules hold a numbered rule for the expiry transition (found rules {exp:?})"));
    report.check("AC3", !expl.is_empty(), format!("the receiver rules hold a numbered rule for the explicit close (found rules {expl:?})"));
    let three = open.iter().any(|o| exp.iter().any(|e| expl.iter().any(|x| o != e && e != x && o != x)));
    report.check("AC3", three, "the open, expiry and explicit-close transitions are three different receiver rules (C-153: announced, modules stamp it while open, closes at expiry or on explicit close)");
}

/// The second announcement (AC3), checked over the receiver rules'
/// paragraphs and the invalid forms.
pub(crate) fn second(s: &Section, rows: &[(String, String)], report: &mut Report) {
    let rul_paras = prose_blocks(&s.rul).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text);
    let rows_joined = rows.iter().map(|(f, o)| format!("{f} {o}"));
    let hit = rul_paras.chain(rows_joined).any(|x| has_i(r"second|another announcement|already open|while (a |the )?window is open", &x) && has_i("replac|ignor|extend|keep|discard|stays|unchanged|restart", &x));
    report.check("AC3", hit, "a second announcement while a window is open has a stated outcome (replace, ignore or extend)");
}

/// AC3's transitions and reviews F2 and F7 (in the script's order: they
/// run after the DoD tags and the open items).
pub(crate) fn ac3(s: &Section, report: &mut Report) {
    transitions(s, report);
}

/// Reviews F2 and F7.
pub(crate) fn f2_f7(s: &Section, report: &mut Report) {
    let rul: Vec<String> = prose_blocks(&s.rul).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).collect();
    let open: Vec<String> = open_items(&s.text, OpenItemPolicy::Leading).into_iter().map(|i| i.text).collect();
    let two = |x: &str| {
        has(r"(?i)\btwo\b", x) && has(r"(?i)windows?\b", x) && has_i("at once|different|same time", x) && has(r"(?i)\bmodules?\b", x) && !has_i("falsifier:|owner:", x)
    };
    report.check("AC3", rul.iter().chain(open.iter()).any(|x| two(x) && has_tag(x)), "the section states whether two modules can hold different windows at once (Work 3 stated outcomes; C-153) (review F2)");
    let unbounded = prose_blocks(&s.text).iter().filter(|b| b.kind == BlockKind::Prose).any(|b| has_i("time state|unusable", &b.text) && has_i(r"\blost\b|\bnever\b", &b.text) && has_i(r"\bclose", &b.text));
    report.check("AC3", unbounded, "the case of a module whose time is unusable and whose close was lost is stated (Work 3 stated outcomes; C-153 closes on explicit close) (review F7)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_and_parts() {
        let r = parse_rules("1. **Open.** A window opens. More.\ncont\n\n**Rule 2** x\n");
        assert_eq!(r, vec!["1. **Open.** A window opens. More. cont", "**Rule 2** x"]);
        assert_eq!(rule_parts(&r[0]), ("open".into(), "a window opens.".into()));
    }
}
