//! The close-out paragraphs: the loss-accounting open item, AC 6 (regime
//! stamp), AC 7's report side (the recorded fingerprint, the re-check
//! paragraph), the hand-offs, the 002-10 open item and the DoD open-item
//! rule.

use super::{POC_TAG_TREE_FINGERPRINT, Rep, TAG, bad, blocks, cut, flat, para_blocks};
use crate::doccheck::Report;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::text::{has, has_i, rx, starts};

/// The loss-accounting open item (lead ruling).
pub(super) fn loss_open_item(rep: &Rep, report: &mut Report) {
    let open: Vec<String> = para_blocks(&rep.lines)
        .into_iter()
        .filter(|p| starts(r"(?i)^\W*Open item", p.py_strip()) && has_i("loss", p))
        .collect();
    if open.is_empty() {
        bad(report, "no paragraph starting \"Open item\" about the POC loss accounting (owner and falsifier required)".into());
    } else if !open
        .iter()
        .any(|p| has_i("owner", p) && has_i("falsifier", p))
    {
        bad(
            report,
            "the open item about the POC loss accounting names no owner or no falsifier".into(),
        );
    } else {
        report
            .passes
            .push(("loss".into(), "open item with owner and falsifier".into()));
    }
}

/// AC 6: the regime stamp is stated not applicable, with its reason.
pub(super) fn ac6(rep: &Rep, report: &mut Report) {
    let ok = para_blocks(&rep.lines).iter().any(|p| {
        has_i("regime stamp", p)
            && has_i("not applicable", p)
            && has_i("because|no stamp field|no runtime code", p)
    });
    if ok {
        report.passes.push(("AC6".into(), "regime stamp".into()));
    } else {
        bad(report, "AC 6: no paragraph states \"regime stamp\" + \"not applicable\" + a reason (because / no stamp field / no runtime code)".into());
    }
}

/// `hash_for(pattern)`: the first 64-hex run on the first line matching
/// `pattern` (`re.I`), or on the line after it.
fn hash_for(rep: &Rep, pattern: &str) -> Option<String> {
    let hex = rx("([0-9a-fA-F]{64})");
    let i = rep.lines.iter().position(|l| has_i(pattern, l))?;
    let on = |k: usize| {
        rep.lines
            .get(k)
            .and_then(|l| hex.captures(l).map(|m| m[1].to_string()))
    };
    on(i).or_else(|| on(i + 1))
}

/// AC 7, the report's side. The recomputation from the proof of concept's
/// tag, the "cannot recompute" failure and the 27-file manifest are in
/// `scripts/repo-state.d/002-13.sh`; `computed` is the frozen tag's
/// [`POC_TAG_TREE_FINGERPRINT`], which that file recomputes.
pub(super) fn ac7(rep: &Rep, report: &mut Report) {
    let computed = POC_TAG_TREE_FINGERPRINT;
    match hash_for(rep, "poc-untouched tag-tree fingerprint") {
        None => bad(
            report,
            "AC 7: the Environment section records no 64-hex poc-untouched tag-tree fingerprint"
                .into(),
        ),
        Some(recorded) if recorded.to_lowercase() != computed => bad(
            report,
            format!(
                "AC 7: frozen-tree fingerprint changed (recorded={recorded} computed={computed})"
            ),
        ),
        Some(_) => report
            .passes
            .push(("AC7".into(), "recorded fingerprint".into())),
    }
    let h01 = hash_for(rep, "frozen[- ]tree fingerprint, taken");
    let recheck = para_blocks(&rep.lines).iter().any(|p| {
        has_i("re-?check|re-?checked|re-?verified|recheck", p)
            && has_i("fingerprint", p)
            && has_i("unchanged|identical|match", p)
            && (p.contains(computed) || h01.as_deref().is_some_and(|h| p.contains(h)))
    });
    if recheck {
        report.passes.push(("AC7".into(), "re-check".into()));
    } else {
        bad(report, "AC 7: no paragraph states the re-check of the frozen-tree fingerprint (with \"fingerprint\", \"unchanged\"/\"identical\"/\"match\" and the value)".into());
    }
}

/// A hand-off: its predicate on a block, and its label.
type HandOff = (fn(&str) -> bool, &'static str);

/// The three hand-offs.
fn hand_off_specs() -> [HandOff; 3] {
    [
        (
            |b| has(r"\b003\b", b) && has_i("confirm", b) && has(r"\(a\)", b),
            "hand-off to 003 (bugs confirmed, the (a) answer)",
        ),
        (
            |b| {
                b.contains("007")
                    && b.contains("017")
                    && has_i("exact.tier", b)
                    && has_i("analyse", b)
            },
            "hand-off to 007 and 017 (exact-tier baseline, what analyse compares)",
        ),
        (
            |b| has(r"\b011\b", b) && b.contains("I/O") && has_i("inherit|dropped", b),
            "hand-off to 011 (I/O-pressure rule inherited or dropped)",
        ),
    ]
}

/// The hand-offs.
pub(super) fn hand_offs(rep: &Rep, report: &mut Report) {
    let Some(hi) = rep.heading_index("Hand-off") else {
        bad(report, "Hand-offs: no heading containing \"Hand-off\"".into());
        return;
    };
    let hj = rep.section_end(hi, rep.level[hi]);
    let hb = blocks(&rep.lines[hi + 1..hj]);
    for (pred, label) in hand_off_specs() {
        match hb.iter().find(|b| pred(b)) {
            None => bad(report, format!("Hand-offs: missing {label}")),
            Some(b) if !has(TAG, b) => bad(
                report,
                format!("Hand-offs: the {label} carries no evidence tag"),
            ),
            Some(_) => report.passes.push(("Hand-offs".into(), label.into())),
        }
    }
}

/// The 002-10 open item (run-1 degradation): closed with the sign-test
/// counts, or re-owned outside the epic with a falsifier.
pub(super) fn open_002_10(rep: &Rep, report: &mut Report) {
    let Some(s10) = rep.heading_index(r"^#+\s*.*002-10") else {
        bad(
            report,
            "002-10 open item: the 002-10 subsection was not found".into(),
        );
        return;
    };
    let e10 = rep.section_end(s10, 3);
    let cands: Vec<String> = para_blocks(&rep.lines[s10 + 1..e10])
        .into_iter()
        .filter(|p| has_i("run 1", p) && has_i("sign|falsifier|open item|undecided|owner", p))
        .collect();
    let Some(p) = cands.first() else {
        bad(
            report,
            "002-10 open item: no run-1 degradation paragraph found in the 002-10 subsection"
                .into(),
        );
        return;
    };
    let flat = flat(p);
    let still_open = has_i("open item|undecided|owner", &flat);
    let closed = !still_open
        && has(r"(^|\W)F\s*=\s*\d+", &flat)
        && has(r"(^|\W)B\s*=\s*\d+", &flat)
        && has(TAG, &flat)
        && has_i("sign[- ]test", &flat);
    let reowned = rx(r"(?i)owner[^.;]*").find(&flat).is_some_and(|mo| {
        let mo = mo.as_str();
        has(r"\b\d{3}-\d{2}\b|milestone", mo) && !mo.contains("002") && has_i("falsifier", &flat)
    });
    if closed || reowned {
        report
            .passes
            .push(("002-10 open item".into(), "closed or re-owned".into()));
    } else {
        bad(report, format!("002-10 open item (run-1 degradation) is still owned by this epic / 002-13 with no sign-test result: current text: {}", cut(&flat, 1500)));
    }
}

/// DoD: the first open-item paragraph without an owner or a falsifier.
pub(super) fn dod(rep: &Rep, report: &mut Report) {
    let offender = para_blocks(&rep.lines).into_iter().find(|p| {
        has_i("open item", p) && !(has_i("owner", p) && has_i("falsifier", p))
    });
    match offender {
        Some(p) => bad(
            report,
            format!(
                "DoD: an open-item paragraph names no owner or no falsifier: {}",
                cut(&flat(&p), 120)
            ),
        ),
        None => report
            .passes
            .push(("DoD".into(), "every open item owned".into())),
    }
}
