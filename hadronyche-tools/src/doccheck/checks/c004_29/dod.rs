//! DoD: evidence tags, owned open items, the reserved fields of the new
//! messages, Open item 11 of the regime stamp; review F1, a denial that the
//! message counter is a liveness counter.

use super::forms::all_paras;
use super::{Merged29, Section};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_27::{Ctx, Field};
use crate::doccheck::evidence::{TagPolicy, has_tag, untagged};
use crate::doccheck::items::{OpenItemPolicy, item_is_owned, open_items};
use crate::doccheck::text::{has, has_i, head, starts};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// Open item 11 of the regime stamp: (the paragraph, its status form).
fn oi11(ctx: &Ctx, s: &Section) -> (String, bool, bool) {
    let stamp = ctx.body(ctx.span_of(r"^regime\s+stamp$", 2));
    let p = prose_blocks(&stamp).into_iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text).find(|x| starts(r"\s*\**\s*Open item 11\b", x)).unwrap_or_default();
    let status = has_i("status", &p) && has_i("commissioning window messages|004-29", &p) && has("64|width", &p);
    let open = p.contains("ticket 004-29 (commissioning window announcement and close messages)")
        && open_items(&s.text, OpenItemPolicy::Leading).iter().any(|i| i.text.contains("Open item 11"));
    (p, status, open)
}

/// The DoD; returns `oi11_status`.
pub(crate) fn dod(ctx: &Ctx, s: &Section, m: &Merged29, new_fields: &[&Field], report: &mut Report) -> bool {
    for (title, txt) in [("announcement", &s.ann), ("close", &s.clo), ("receiver rules", &s.rul)] {
        report.check("DoD", !txt.is_empty() && has_tag(txt), format!("the {title} subsection carries an evidence tag"));
    }
    let bad: Vec<String> = untagged(&s.text, &TagPolicy::PROSE_004_29).iter().map(|u| head(&u.text, 50)).collect();
    report.check("DoD", bad.is_empty(), format!("every prose paragraph of 5 or more words in the section carries an evidence tag; untagged: {bad:?}"));
    let open = open_items(&s.text, OpenItemPolicy::Leading);
    report.check("DoD", open.iter().all(|i| item_is_owned(i, OpenItemPolicy::Leading)), format!("every open item of the section names an owner and a falsifier ({} found)", open.len()));
    let needs = new_fields.iter().filter(|f| f.off >= 32 && has_i("reserved|unused|accept[- ]any|ignored|0 only", &format!("{} {}", f.meaning, f.valid))).count();
    let res = m.res_rows.iter().filter(|r| r.cells.len() > 1 && has_i("commissioning|004-29", r.cell(1))).count();
    report.check("DoD", needs == 0 || res > 0, format!("every reserved or accept-any field of the new messages ({needs} found) has a row in the merged Reserved fields table ({res} rows name this ticket)"));
    let (p, status, open11) = oi11(ctx, s);
    report.check("DoD", !p.is_empty() && (status || open11), "Open item 11 of the regime stamp is no longer open where this text decides it: a status sentence names this section (a), or the item is unchanged and a new open item here names it (b)");
    status
}

/// Review F1.
pub(crate) fn f1(s: &Section, report: &mut Report) {
    let deny = r"(?i)cannot be a (?:data-plane )?liveness counter|not a (?:data-plane )?liveness counter|cannot be the liveness counter";
    for x in all_paras(s).into_iter().filter(|x| has(deny, x)) {
        let framed = has_i("reported contradiction|present disagreement", &x) || starts(r"\s*\**\s*Open item", &x);
        let ok = framed && x.contains("Hello") && x.contains("C-115") && has_i("owner", &x) && has_i("falsifier", &x);
        report.check("DoD", ok, "a denial that the tool's message counter is a liveness counter is a reported contradiction or an open item, naming the merged Hello row and C-115, with an owner and a falsifier (review F1; Hello, Common control header, offset 8; C-115)");
    }
}
