//! AC3: the sender and receiver rules, and the review findings (first
//! stamp, restarted peer, fetch only when stale, the fetch-load open item).

use super::landed::Landed;
use super::py::sentences_lower;
use super::stamp::Stamp;
use crate::doccheck::Report;
use crate::doccheck::evidence::has_tag;
use crate::doccheck::text::{has, has_i, head};

fn sender_receiver(st: &Stamp, landed: &Landed, report: &mut Report) {
    report.check("AC3", st.any_sent(&["multicast", "change"]), "a sender rule says the stamp is multicast to the control group on any regime change (C-18)");
    // Python formats None as "None" into the pattern; it can only match the
    // literal text "None s", which no sentence (lowered) contains.
    let refresh = match landed.refresh_s {
        Some(n) => format!(r"(?:^|[^\d.]){n} ?(s|seconds)\b"),
        None => r"(?:^|[^\d.])None ?(s|seconds)\b".to_string(),
    };
    report.check("AC3", st.any_sent(&[&refresh, "refresh|every|period|interval"]), format!("a sender rule says the stamp is refreshed every {:?} s (C-18)", landed.refresh_s));
    report.check("AC3", st.any_sent(&["unicast", "request|fetch"]), "a request is unicast and gives an immediate fetch of the stamp (C-18)");
    report.check("AC3", st.any_sent(&["version", "bump|increment|advance|change"]), "the sender bumps the stamp version on any change of its stamp");
    report.check("AC3", st.any_sent(&["unknown|not know|does not know|unrecogni", "skip|ignore", "length"]), "a receiver skips an unknown section type of the same major by its length (004-04)");
    report.check("AC3", st.any_sent(&["incomplete", "discard|drop|replace|abandon", "newer|later|higher|new version|greater"]), "a receiver discards an incomplete stamp when a newer version arrives");
    report.check("AC3", st.any_sent(&["complete", "version"]), "a receiver can tell that it holds a complete stamp of one version (Work item 1)");
    let parts = st.subs.iter().any(|(t, _)| has_i("send", t)) && st.subs.iter().any(|(t, _)| has_i("receiv", t));
    report.check("AC3", parts, "the stamp section has a sender part and a receiver part (Work items 4 and 5)");
    let ret = st.paras().filter(|t| has_i("unicast|multicast", t) && has_i("request|fetch", t) && has_i("return|reply|answer|sent back|respon", t)).any(|t| t.contains("[decision]"));
    report.check("AC3", ret, "a paragraph says how the stamp is returned to the requester (unicast or multicast) and is tagged [decision] (Work item 3)");
}

fn review_findings(st: &Stamp, report: &mut Report) {
    let tagged: Vec<String> = st.paras().filter(|t| has_tag(t)).flat_map(|t| sentences_lower(t, ".;:")).collect();
    let first = tagged.iter().any(|x| has(r"\bfirst\b|nothing held|no stamp held|none held|nothing is held", x) && has("section|stamp", x) && has(r"\baccept|\bstart|\bbegin", x));
    report.check("AC3", first, "a tagged receiver rule gives the outcome (accept, start or begin) for the first section of a peer when nothing is held or reassembled");
    let nonce = tagged.iter().any(|x| x.contains("nonce") && has(r"\bdifferent\b|\bnew\b|restart", x) && has("discard|replace|drop", x));
    report.check("AC3", nonce, "a tagged receiver rule says that a section with a different instance nonce from the held stamp's discards or replaces the old stamp");
    let stale = tagged.iter().any(|x| x.contains("unicast") && has(r"\bonly\b|\bsolely\b|exactly when", x) && has("stale|differ", x) && has("fetch|request", x));
    report.check("AC3", stale, "a tagged rule says a receiver fetches by unicast only when its held copy is stale (the pair differs)");
    let fetch_open = st.paras().filter(|t| has_i("open item", t) && has_i("request|fetch|load", &head(t, 200))).any(|t| t.contains("011-23") && t.contains("005-07"));
    report.check("AC3", fetch_open, "the open item about the fetch load names 011-23 and 005-07 as owners");
}

/// AC3.
pub(crate) fn ac3(st: &Stamp, landed: &Landed, report: &mut Report) {
    sender_receiver(st, landed, report);
    review_findings(st, report);
}
