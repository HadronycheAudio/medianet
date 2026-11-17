//! DoD: evidence tags (prose and subsections with a table), owned open
//! items, and the review points labelled DoD (point 2, the major-change
//! sentence; T3 (c), no [measured] maximum and [decision] on chosen
//! numbers; T3 (d), reserved fields recorded; T3 (g), what waits for every
//! section).

use std::collections::BTreeMap;

use super::fields::Man;
use super::forms::open_paras;
use super::{Wire, is_field_table, subs};
use crate::doccheck::Report;
use crate::doccheck::pycompat::PyStr;
use crate::doccheck::family004::tables;
use crate::doccheck::sentences::{RAW_COLON, split_raw};
use crate::doccheck::evidence::{TagPolicy, subsections_with_table_untagged, untagged};
use crate::doccheck::text::{has, has_i, head};


/// Evidence tags on prose and on subsections with a table; owned open
/// items.
pub(crate) fn tags(man: &Man, report: &mut Report) {
    for u in untagged(&man.text, &TagPolicy::PROSE_004_26) {
        report.fail("DoD", format!("prose paragraph '{}...' carries an evidence tag", head(u.text.py_strip(), 50)));
    }
    for title in subsections_with_table_untagged(&man.lines.join("\n")) {
        report.fail("DoD", format!("subsection '{title}' with a table carries an evidence tag"));
    }
    for p in open_paras(man) {
        report.check("DoD", has_i(r"\bowner\b", &p) && has_i(r"\bfalsifier\b", &p), format!("open item '{}...' names an owner and a falsifier", head(p.py_strip(), 50)));
    }
}

/// Review point 2: the major-change sentence excepts a new content type.
pub(crate) fn point2(man: &Man, report: &mut Report) {
    for p in &man.paras_flat {
        for s in split_raw(p, RAW_COLON) {
            if has_i("every addition", s) && has_i(r"\bmajor\b", s) {
                report.check("DoD", has_i("content[- ]type", s) && has_i(r"\bexcept\b|other than", s), "the sentence that says every addition to the manifest is a major change excepts a new content-type code (accept-any)");
            }
        }
    }
}

/// Review T3 (c): no [measured] maximum, [decision] on chosen numbers.
pub(crate) fn t3c(man: &Man, report: &mut Report) {
    let maxw = r"maximum|at[ -]most|\bmax\b|no more than|\bup to\b|\bto\s+\d";
    let sents = man.para_sentences();
    let bad_s = sents.iter().any(|s| s.contains("[measured]") && has_i(maxw, s));
    let bad_r = man.all_rows().any(|r| {
        let t = format!("{} {}", r.meaning, r.valid);
        t.contains("[measured]") && has_i(maxw, &t)
    });
    report.check("DoD", !bad_s && !bad_r, "no stated maximum carries the [measured] tag");
    let chosen: Vec<&String> = sents.iter().filter(|s| has_i(r"\bchosen\b|not derived", s) && has_i(maxw, s) && has(r"\d", s)).collect();
    report.check("DoD", chosen.iter().all(|s| s.contains("[decision]")), format!("every sentence that calls a number chosen or not derived is tagged [decision] ({} found)", chosen.len()));
}

/// Review T3 (g): what waits for every section.
pub(crate) fn t3g(man: &Man, report: &mut Report) {
    let wait = r"(?i)(judges?|wait|waits|waiting|decide|decides|hold|holds)[^.]{0,120}(every section|complete manifest)";
    let except = r"(?i)\bexcept\b|other refusals|without (the|a|its) manifest|(does|do|need) not wait";
    for p in man.paras_flat.iter().filter(|p| has(wait, p)) {
        report.check("DoD", p.contains("BadMemberList") && has(except, p), format!("the paragraph that waits for every section names BadMemberList as waiting and what does not wait ('{}...')", head(p, 50)));
    }
}

/// Review T3 (d): reserved fields recorded.
pub(crate) fn reserved(man: &Man, wire: &Wire, report: &mut Report) {
    let res = wire.section_lines(r"^Reserved fields$", 2).unwrap_or_default();
    let res_rows: Vec<Vec<String>> = tables(&res.join("\n"))
        .into_iter()
        .filter(|t| t.head().first().is_some_and(|h| h.to_lowercase().starts_with("field")))
        .flat_map(|t| t.rows.into_iter().map(|r| r.cells))
        .collect();
    let mut title_of: BTreeMap<String, String> = BTreeMap::new();
    for (title, body) in subs(&man.lines) {
        for t in tables(&body).into_iter().filter(|t| is_field_table(t.head())) {
            for r in t.rows.iter().filter(|r| r.cells.len() >= 5) {
                title_of.insert(r.cell(3).py_strip().to_string(), title.to_lowercase());
            }
        }
    }
    let reserved = man.all_rows().filter(|r| r.meaning.starts_with("Reserved") || r.valid.py_strip().to_lowercase() == "0 only" || has_i("must be zero|accept any|ignores? any other", &r.valid));
    for r in reserved {
        let title = title_of.get(&r.meaning).cloned().unwrap_or_default();
        let key = title.split_whitespace().next().unwrap_or("").to_string();
        let mut words = vec!["manifest".to_string(), key];
        if has_i("content", &format!("{}{}", r.meaning, r.valid)) {
            words.push("content".into());
        }
        let found = res_rows.iter().filter(|rr| rr.len() >= 2).any(|rr| {
            let hay = format!("{} {}", rr[0], rr[1]).to_lowercase();
            words.iter().all(|w| !w.is_empty() && hay.contains(w.as_str()))
        });
        report.check("DoD", found, format!("the reserved or accept-any manifest field '{}' has a row in the landed Reserved fields table naming {words:?}", head(&r.meaning, 50)));
    }
}
