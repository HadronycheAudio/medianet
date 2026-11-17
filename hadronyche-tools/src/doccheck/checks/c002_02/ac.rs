//! The acceptance criteria and the DoD item of 002-02, each as the script
//! computes it (bash semantics from `family002`).

use super::{Sub, verdict};
use crate::doccheck::Report;
use crate::doccheck::family002::{
    Grep, Lines, Sed, alnum_word_count, cut_bytes, distinct_count, heredoc_lines, lstrip_space,
    rstrip_space, strip_prefix_once, strip_suffix_once, tr_delete_space, tr_lower,
};

const POINTER: &str = r"audio\.rs:[0-9]+";
const MARKER: &str = r"(^|[^[:alnum:]])(confirmed|corrected)([^[:alnum:]]|$)";
const LIST_ENTRY: &str = r"^[[:space:]]*(-|\*|\+|>|[0-9]+[.)])[[:space:]]|^[[:space:]]*\|";
const REASON: &str = "because|since|reason|so that|to avoid|otherwise";

/// The script's `normalise`: leading `[[:space:]]` removed; then each of
/// `- `, `* `, `+ `, `> ` removed once, in that order; leading space again;
/// one `**` removed at each end; leading space again.
pub(crate) fn normalise(s: &str) -> &str {
    let mut s = lstrip_space(s);
    for p in ["- ", "* ", "+ ", "> "] {
        s = strip_prefix_once(s, p);
    }
    s = lstrip_space(s);
    s = strip_suffix_once(strip_prefix_once(s, "**"), "**");
    lstrip_space(s)
}

/// AC1: at most 45 non-empty lines; at least 4 distinct `audio.rs:N`.
pub(crate) fn ac1(sub: &Sub, r: &mut Report) {
    let nonempty = Grep::bre("[^[:space:]]").count_lines(&sub.text);
    verdict(
        r,
        "AC1",
        nonempty <= 45,
        format!("sub-section is compact ({nonempty} non-empty lines, limit 45)"),
        format!("sub-section is too long ({nonempty} non-empty lines, limit 45 — about one page)"),
    );
    let nptr = distinct_count(Grep::ere(POINTER).only_lines(&sub.text));
    verdict(
        r,
        "AC1",
        nptr >= 4,
        format!("sub-section carries {nptr} distinct audio.rs:NNN pointers"),
        format!("sub-section has {nptr} distinct audio.rs:NNN pointer(s), need at least 4"),
    );
}

/// AC2: the exact bit-level answer; a "no" names the tolerance.
pub(crate) fn ac2(sub: &Sub, r: &mut Report) {
    let answer = heredoc_lines(&sub.text).into_iter().find_map(|raw| {
        let nl = tr_lower(normalise(raw));
        match rstrip_space(&nl) {
            "voice comparison bit-level: yes" => Some("yes"),
            "voice comparison bit-level: no" => Some("no"),
            _ => None,
        }
    });
    let Some(answer) = answer else {
        r.fail(
            "AC2",
            "missing an exact line 'Voice comparison bit-level: yes' or '...: no' (markdown emphasis and list bullets are tolerated)",
        );
        return;
    };
    r.check("AC2", true, format!("bit-level question answered exactly: 'Voice comparison bit-level: {answer}'"));
    if answer == "no" {
        let named = Grep::ere_i(r"toleran|threshold|epsilon|eps\b").any_line(&sub.text);
        verdict(
            r,
            "AC2",
            named,
            "answer is 'no' and the sub-section names the tolerance used".into(),
            "answer is 'no' but the sub-section names no tolerance (need one of: tolerance, threshold, epsilon/eps)".into(),
        );
    }
}

/// The five loss-handling facts: label, anchor, needles, what the
/// not-stated message asks for.
const FACTS: [(&str, &str, &[&str], &str); 5] = [
    ("1024-sample comparison window", "1024|1[ ,]024", &["1024|1[ ,]024", "window"], "need '1024' and 'window'"),
    ("window compared only when complete on both sides", "complete", &["complete", "both"], "need 'complete' and 'both'"),
    (
        "256-window lag drop (MAX_LAG_WINDOWS, about 2.7 s)",
        "256|MAX_LAG_WINDOWS",
        &["256|MAX_LAG_WINDOWS", "2[.,]7"],
        "need '256' and the '2.7' s figure on one statement",
    ),
    (
        "1-second silence gives a 3-second settling exclusion",
        "settle|settling",
        &["settle|settling", "(1[ -]?s|1 second|1-second)", "(3[ -]?s|3 second|3-second|SETTLE_SECS)"],
        "need settling, a 1-second silence and a 3-second exclusion",
    ),
    ("sender's replica never applies gaps", "replica", &["replica", "gap"], "need 'replica' and 'gap'"),
];

/// AC3: each fact stated (anchor line + the next line of the FILE hold every
/// needle) and that block marked confirmed/corrected with `[code]`.
pub(crate) fn ac3(sub: &Sub, r: &mut Report) {
    for (label, anchor, needles, need) in FACTS {
        let anchor = Grep::ere_i(anchor);
        let found = (sub.start + 1..=sub.end).find(|k| {
            anchor.matches(sub.lines.sed_line(*k)) && {
                let block = sub.lines.line_range(*k, k + 1);
                needles.iter().all(|n| Grep::ere_i(n).any_line(&block))
            }
        });
        let Some(k) = found else {
            r.fail("AC3", format!("fact '{label}' not stated ({need})"));
            continue;
        };
        let block = sub.lines.line_range(k, k + 1);
        let marked = Grep::ere_i(MARKER).any_line(&block) && Grep::fixed("[code]").any_line(&block);
        verdict(
            r,
            "AC3",
            marked,
            format!("fact marked confirmed/corrected [code]: {label}"),
            format!("fact '{label}' is stated (line {k}) but not marked confirmed/corrected with [code]"),
        );
    }
}

/// The five exclusion categories: label, ERE.
const EXCLUSIONS: [(&str, &str); 5] = [
    ("window incomplete (not complete on both sides)", "incomplete"),
    ("lag over MAX_LAG_WINDOWS", "lag|MAX_LAG_WINDOWS"),
    ("sender gap", "sender[ -]?gap"),
    ("SETTLE_SECS settling", "settle|SETTLE_SECS"),
    ("voices flag off", "voices[ -]?flag|voices.*off|flag off|voices=false|voices = false"),
];

/// AC4: every list entry naming an exclusion (and not marked
/// confirmed/corrected) has a pointer and a reason; each category has a
/// complete entry.
pub(crate) fn ac4(sub: &Sub, r: &mut Report) {
    let entry = Grep::ere(LIST_ENTRY);
    let pointer = Grep::ere(POINTER);
    let reason = Grep::ere_i(REASON);
    let (mut any, mut bad) = (0, 0);
    for ln in heredoc_lines(&sub.text) {
        if tr_delete_space(ln).is_empty() || !entry.matches(ln) {
            continue;
        }
        if !EXCLUSIONS.iter().any(|(_, re)| Grep::ere_i(re).matches(ln)) || Grep::ere_i(MARKER).matches(ln) {
            continue;
        }
        any += 1;
        let mut missing = String::new();
        if !pointer.matches(ln) {
            missing.push_str(" pointer");
        }
        if !reason.matches(ln) {
            missing.push_str(" reason");
        }
        if !missing.is_empty() {
            bad += 1;
            r.fail("AC4", format!("exclusion entry lacks{missing}: {}", cut_bytes(ln, 110)));
        }
    }
    if any == 0 {
        r.fail(
            "AC4",
            "no exclusion list entry found (expected bullet or table rows naming incomplete/LAG/sender gap/settling/voices flag)",
        );
        return;
    }
    verdict(
        r,
        "AC4",
        bad == 0,
        format!("all {any} exclusion entries carry an audio.rs:NNN pointer and a stated reason"),
        format!("{bad} of {any} exclusion entries lack a pointer or a stated reason"),
    );
    let missing: String = EXCLUSIONS
        .iter()
        .filter(|(_, re)| {
            !heredoc_lines(&sub.text).into_iter().any(|ln| {
                entry.matches(ln) && Grep::ere_i(re).matches(ln) && pointer.matches(ln) && reason.matches(ln)
            })
        })
        .map(|(label, _)| format!("; {label}"))
        .collect();
    verdict(
        r,
        "AC4",
        missing.is_empty(),
        "all five named exclusion categories are covered by a complete entry".into(),
        format!("exclusion categories missing a complete entry (pointer + reason):{missing}"),
    );
}

/// AC5: a `Verdict statement:` line with text, one sentence (at most one
/// terminal mark after removing pointers, decimals, e.g./i.e. and tags; at
/// least 6 words).
pub(crate) fn ac5(sub: &Sub, r: &mut Report) {
    let verdict_text = heredoc_lines(&sub.text)
        .into_iter()
        .find_map(|raw| lstrip_space(normalise(raw)).strip_prefix("Verdict statement:"))
        .unwrap_or("");
    if tr_delete_space(verdict_text).is_empty() {
        r.fail(
            "AC5",
            "missing the required verdict line: a line starting with 'Verdict statement:' followed by the one-sentence statement",
        );
        return;
    }
    r.check("AC5", true, "verdict statement line present");
    let cleaned = Sed::ere(r"s/audio\.rs:[0-9]+//g; s/[0-9]+\.[0-9]+//g; s/\be\.g\.//g; s/\bi\.e\.//g").apply(verdict_text);
    let cleaned = Sed::ere(r"s/\[(code|measured|kernel-doc|memory|decision)\]//g").apply(&cleaned);
    let terms = Grep::ere("[.!?]").only(&cleaned).len();
    let words = alnum_word_count(&cleaned);
    verdict(
        r,
        "AC5",
        terms <= 1 && words >= 6,
        format!("verdict statement is one sentence ({words} words, {terms} terminal mark(s))"),
        format!("verdict statement is not one sentence ({words} words, {terms} terminal mark(s); need <=1 and >=6 words)"),
    );
}

/// DoD: the Open sub-questions row about the bit-level voice comparison is
/// still a table row, marked answered with yes or no.
pub(crate) fn dod(lines: &Lines, r: &mut Report) {
    let Some(q) = lines.heading_no(&Grep::ere_i(r"^#+[[:space:]]*Open sub-questions")) else {
        r.fail("DoD", "no '## Open sub-questions' section found");
        return;
    };
    let wanted = Grep::bre_i("voice comparison bit-level");
    let Some(row) = lines.read_loop_from(q + 1).into_iter().find(|l| wanted.matches(l)) else {
        r.fail("DoD", "sub-questions table has no row about the voice comparison being bit-level");
        return;
    };
    r.check("DoD", true, "sub-questions table still has the voice-comparison row");
    if !Grep::ere(r"^[[:space:]]*\|").matches(row) {
        r.fail("DoD", "the voice-comparison row is no longer a table row (must still start with '|')");
    }
    if !Grep::bre_i("answered").matches(row) {
        r.fail("DoD", "the voice-comparison row is not marked answered (must contain 'answered' plus yes/no)");
        return;
    }
    verdict(
        r,
        "DoD",
        Grep::ere_i(r"(^|[^[:alnum:]])(yes|no)([^[:alnum:]]|$)").matches(row),
        "the voice-comparison row is marked answered with yes/no".into(),
        "the voice-comparison row says 'answered' but does not state yes or no".into(),
    );
}
