//! AC1 and AC2: the announcement and the close — their tables
//! (`msg_analysis`), the window identifier, the tool identity, the expiry's
//! width, unit, form and clock (read from timeline.md at run time).

use super::clock::clock_terms;
use super::{Merged29, Section};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::first_table;
use crate::doccheck::checks::c004_25::types::stated_sizes;
use crate::doccheck::checks::c004_27::{Field, fields_of, is_field_table};
use crate::doccheck::evidence::has_tag;
use crate::doccheck::text::{has, has_i, rx, starts};
use crate::doccheck::units::{BlockKind, prose_blocks};

/// What `msg_analysis` returns.
pub(crate) struct Msg {
    pub fl: Vec<Field>,
    pub end: i64,
    pub code: Option<i64>,
}

/// `msg_analysis(text, title, label)`; `Err` where the script stops.
pub(crate) fn msg_analysis(text: &str, title: &str, label: &str, m: &Merged29, report: &mut Report) -> Result<Option<Msg>, Stop> {
    let Some(tb) = first_table(text, is_field_table) else {
        report.fail(label, format!("the {title} has no field table (Offset, Width, Type, Meaning, Valid range)"));
        return Ok(None);
    };
    let fl = fields_of(Some(&tb), label, title, report);
    let hdr: Vec<(i64, i64)> = fl.iter().take(7).map(|f| (f.off, f.bits)).collect();
    report.check(label, fl.len() >= 7 && hdr == m.cch_rows, format!("the {title} starts with the seven rows of the common control header of the Hello section (read {hdr:?})"));
    let (mut pos, mut tiles) = (0, true);
    for f in &fl {
        if f.bits % 8 != 0 || f.off != pos {
            tiles = false;
        }
        pos = f.off + f.bits.div_euclid(8);
    }
    report.check(label, tiles, format!("the {title} table tiles from byte 0 with no gap or overlap (its rows end at byte {pos})"));
    let stated = stated_sizes(text);
    let within = match (stated.contains(&pos), m.ctrl_bound) {
        (true, None) => return Err(Stop("TypeError: int <= None (C-17 unread)".into())),
        (s, b) => s && b.is_some_and(|b| pos <= b),
    };
    report.check(label, within, format!("the {title} states its maximum size as its table sum, {pos} bytes, within the control bound (C-17) of {:?} bytes (stated {stated:?})", m.ctrl_bound));
    let mut code = None;
    for f in fl.iter().filter(|f| f.off == 6) {
        code = rx(r"^\s*(\d+)\s+only").captures(&f.valid).and_then(|c| c[1].parse().ok());
    }
    let inside = matches!((code, m.block), (Some(c), Some((lo, hi))) if lo <= c && c <= hi);
    report.check(label, inside, format!("the {title} has a type code, in its Message type row, inside the block the Hello section gives this ticket ({:?}) (found {code:?})", m.block));
    Ok(Some(Msg { fl, end: pos, code }))
}

const EXTRA_CLOCKS: [&str; 3] = ["wall clock", "utc", "tool's clock"];
const UNIT: &str = r"(?i)nanosecond|microsecond|millisecond|\bns\b|µs|\bus\b|\bms\b|\bsecond";

fn tagged_paras(text: &str) -> Vec<String> {
    prose_blocks(text).into_iter().filter(|b| b.kind == BlockKind::Prose && has_tag(&b.text)).map(|b| b.text).collect()
}

fn clock(s: &Section, ann: &Msg, timeline: &str, report: &mut Report) {
    let terms = clock_terms(timeline);
    let shared: Vec<&String> = terms.iter().filter(|t| t.contains("network time")).collect();
    let eval_re = r"(?i)evaluat|convert|does not share|do not share|without the holder|on its own clock|maps? it";
    let tagged = tagged_paras(&s.ann);
    let eval_paras = tagged.iter().any(|x| has(eval_re, x));
    let src = ann.fl.iter().map(|f| format!("{} {}", f.meaning, f.valid)).chain(tagged.iter().cloned()).collect::<Vec<_>>().join(" ").to_lowercase();
    let word = |t: &str| rx(&format!(r"\b{}\b", regex::escape(t))).is_match(&src);
    let named: Vec<&String> = terms.iter().filter(|t| word(t)).collect();
    let extra = EXTRA_CLOCKS.iter().any(|t| word(t));
    let named_shared = named.iter().any(|t| shared.contains(t));
    let named_local = named.iter().any(|t| !shared.contains(t));
    report.check("AC2", named_shared || ((named_local || extra) && eval_paras), format!("the announcement names its clock: a clock timeline.md defines (network time), or a holder-local or other clock with a tagged sentence on how a module that does not share it evaluates the expiry (Work 1, C-153) (named {named:?})"));
}

fn expiry(s: &Section, ann: &Msg, m: &Merged29, exp: &Field, report: &mut Report) -> bool {
    let ev = format!("{} {}", exp.meaning, exp.valid);
    report.check("AC2", has(UNIT, &ev), "the expiry row states its unit (Work 1; C-153)");
    let rel = has_i("relative|duration", &ev);
    let absf = has_i(r"\babsolute\b", &ev);
    report.check("AC2", rel != absf, "the expiry row states one form: an absolute instant or a relative duration (Open item 11)");
    let form = if rel { "relative" } else { "absolute" };
    report.check("AC2", tagged_paras(&s.ann).iter().any(|x| has_i("expir", x) && has_i(form, x)), format!("a tagged sentence of the announcement states the expiry form ({form}) (review T2)"));
    if !rel {
        return false;
    }
    let both = format!("{ev} {}", s.ann);
    report.check("AC2", has_i("counted from|counts from|from the issue time|from the instant", &both), "the relative expiry states the instant it counts from (Open item 11, branch b)");
    report.check("AC2", has_i(r"\blate\b|twice", &s.ann) && has_i("same expiry|derives? the same", &s.ann), "the relative expiry states how a late or repeated announcement derives the same expiry (Open item 11, branch b)");
    let cw = m.cw.iter().find(|f| starts("(?i)expiry time", &f.meaning));
    let merged_ok = cw.is_some_and(|f| !f.valid.contains("copied from the announcement") && has_i("deriv", &format!("{} {}", f.meaning, f.valid)));
    report.check("AC2", merged_ok, "the merged commissioning window section no longer says the stamped expiry is copied from the announcement: it says the stamped value is derived (Open item 11, branch b)");
    let _ = ann;
    true
}

/// The announcement's checks (AC1, AC2); returns `REL_FORM`.
pub(crate) fn announcement(s: &Section, ann: &Msg, m: &Merged29, timeline: &str, report: &mut Report) -> bool {
    let fl = &ann.fl;
    let wid: Vec<&Field> = fl.iter().filter(|f| starts("(?i)window identifier", &f.meaning)).collect();
    let exp: Vec<&Field> = fl.iter().filter(|f| starts("(?i)expiry", &f.meaning)).collect();
    let tool: Vec<&Field> = fl.iter().filter(|f| has_i("tool", &f.meaning) && has_i("identit", &f.meaning) && !starts("(?i)window identifier", &f.meaning)).collect();
    report.check("AC1", wid.len() == 1, format!("the announcement has one window identifier row (found {})", wid.len()));
    report.check("AC1", tool.len() == 1 && tool[0].bits > 0, format!("the announcement has one row for the announcing tool's identity, with a width (found {})", tool.len()));
    report.check("AC1", has_i("unique", &s.ann) && has_i("stable", &s.ann), "the announcement's text states the window identifier stable and unique, so results can reference it (C-178, Work 1)");
    report.check("AC2", wid.len() == 1 && m.w_stamp.is_some_and(|w| wid[0].bits == w), format!("the announcement's window identifier is as wide as the merged stamp's, {:?} bits", m.w_stamp));
    report.check("AC2", exp.len() == 1 && m.e_stamp.is_some_and(|w| exp[0].bits == w), format!("the announcement's expiry is as wide as the merged stamp's expiry, {:?} bits, so the stamp can copy it", m.e_stamp));
    clock(s, ann, timeline, report);
    if exp.len() == 1 { expiry(s, ann, m, exp[0], report) } else { false }
}

/// The close's checks (AC1, AC2).
pub(crate) fn close(clo: &Msg, m: &Merged29, report: &mut Report) {
    let cwid: Vec<&Field> = clo.fl.iter().filter(|f| starts("(?i)window identifier", &f.meaning)).collect();
    report.check("AC1", cwid.len() == 1, format!("the close names the window it closes by its window identifier row (found {})", cwid.len()));
    report.check("AC2", cwid.len() == 1 && m.w_stamp.is_some_and(|w| cwid[0].bits == w), format!("the close's window identifier is as wide as the merged stamp's, {:?} bits", m.w_stamp));
    report.check("AC1", clo.fl.iter().filter(|f| f.off >= 32).count() == 1, "the close lays out only the identifier after the common control header (Work 2)");
}
