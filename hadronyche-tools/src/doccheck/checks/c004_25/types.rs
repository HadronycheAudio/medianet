//! AC1: the seven section types — one subsection each, their tables and
//! entry tables, sizes within the control bound, the section-count
//! statements, the type codes, the arithmetic, and the 64-section bound.

use super::Stamp;
use super::tree::{Node, bad_arithmetic, build_tree, cdiv, entry_nodes, lo_of, maxc, size_node, tile_node};
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::family004::{Field, fields_of, find_sub, is_field_table, tables};
use crate::doccheck::pycompat::parse_int;
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::text::rx;

/// The seven types and the subsection title patterns that find them.
pub(crate) const TYPE_SPECS: [(&str, &str); 7] = [
    ("multi", r"multi[- ]?packet"),
    ("health", r"health|per[- ]connection"),
    ("calibration", r"calibration"),
    ("commissioning", r"commissioning"),
    ("injection", r"injection"),
    ("capability", r"capabilit"),
    ("timestamp", r"timestamp|time stamp"),
];

/// One section type found in the regime stamp section.
#[derive(Debug, Clone, Default)]
pub(crate) struct SType {
    pub key: &'static str,
    pub pat: &'static str,
    pub title: String,
    pub text: String,
    pub fl_all: Vec<Field>,
    /// The tree, when the type has a non-empty first field table.
    pub root: Option<Node>,
    pub sec_max: Option<i128>,
    pub nt: Option<i128>,
    pub body_rel: bool,
    pub m_stated: Vec<i128>,
    pub code: Option<i128>,
}

impl SType {
    /// The rows of every entry table, depth-first.
    pub fn entry_fields(&self) -> Vec<&Field> {
        self.root.as_ref().map(|r| entry_nodes(r).into_iter().flat_map(|n| n.rows.iter().map(|r| &r.f)).collect()).unwrap_or_default()
    }

    /// The list counts of the root's list rows.
    pub fn list_counts(&self) -> Vec<i128> {
        self.root.as_ref().map(|r| r.rows.iter().filter_map(|x| maxc(&x.f)).collect()).unwrap_or_default()
    }
}

/// `(?:⌈|ceil\s*\()\s*(\d+)\s*/\s*(\d+)\s*(?:⌉|\))\s*=\s*(\d+)`, any case.
pub(super) fn statements(text: &str) -> Vec<(i128, i128, i128)> {
    rx(r"(?i)(?:⌈|ceil\s*\()\s*(\d+)\s*/\s*(\d+)\s*(?:⌉|\))\s*=\s*(\d+)")
        .captures_iter(text)
        .filter_map(|c| Some((parse_int(&c[1]).ok()?, parse_int(&c[2]).ok()?, parse_int(&c[3]).ok()?)))
        .collect()
}

/// `TYPE_CODE_RE`: the first type code the text states.
pub(crate) fn type_code(text: &str) -> Option<i128> {
    let c = rx(r"(?i)type code\D{0,20}?(\d+)|\bcode (\d+)\b|section type\D{0,20}?(\d+)\b").captures(text)?;
    (1..=3).find_map(|i| c.get(i)).and_then(|m| parse_int(m.as_str()).ok())
}

/// `SIZE_MAX_RE`: every "maximum … N bytes" figure.
pub(crate) fn stated_sizes(text: &str) -> Vec<i128> {
    rx(r"(?i)maximum[^.;]{0,120}?(\d+)\s*bytes").captures_iter(text).filter_map(|c| parse_int(&c[1]).ok()).collect()
}

fn counts(t: &mut SType, root: &Node, report: &mut Report) {
    let stmts = statements(&t.text);
    let mut nt = 1;
    for f in root.rows.iter().filter_map(|r| maxc(&r.f)) {
        let Some(&(m, _, n)) = stmts.iter().find(|s| s.1 == f) else {
            report.fail("AC1", format!("the {} list row (at most {f} entries per section) has no statement of the form ⌈M / {f}⌉ = N", t.key));
            continue;
        };
        if n != cdiv(m, f) {
            report.fail("AC1", format!("the {} section count is wrong: ⌈{m} / {f}⌉ = {}, the text states {n}", t.key, cdiv(m, f)));
        }
        nt = nt.max(cdiv(m, f));
    }
    t.nt = Some(nt);
    let lists: Vec<i128> = root.rows.iter().filter_map(|r| maxc(&r.f)).collect();
    t.m_stated = stmts.iter().filter(|s| lists.contains(&s.1)).map(|s| s.0).collect();
}

/// The tree of one type: `fl_all`, the root with its starts and sizes.
fn one_type(t: &mut SType, ctrl_bound: Option<i128>, block: Option<(i128, i128)>, report: &mut Report) -> Flow<()> {
    let mut ft = Vec::new();
    for tb in tables(&t.text).iter().filter(|tb| is_field_table(tb.head())) {
        let fl = fields_of(Some(tb), "AC1", &t.title, report);
        t.fl_all.extend(fl.iter().cloned());
        ft.push(fl);
    }
    if ft.first().is_none_or(Vec::is_empty) {
        report.fail("AC1", format!("the {} section type has no field table of Offset, Width, Type, Meaning, Valid range", t.key));
        return Ok(());
    }
    let (mut root, unused) = build_tree(&ft);
    if !unused.is_empty() {
        report.fail("AC1", format!("the {} section type has {} field table(s) that no list row of it names as its entry", t.key, unused.len()));
    }
    let first = &root.rows[0].f;
    let off0 = first.off;
    t.body_rel = off0 != 58;
    if off0 != 0 && off0 != 58 {
        report.fail("AC1", format!("the {} table starts at offset {off0}: a body starts at 0 and a section at 58", t.key));
    }
    let start = checked::add(checked::mul(off0, 8), lo_of(first).unwrap_or_default());
    tile_node(&mut root, start, &t.title, "AC1", report);
    let rbits = size_node(&mut root, &t.title, report);
    let sec_max = checked::add(58, cdiv(rbits, 8));
    t.sec_max = Some(sec_max);
    let Some(b) = ctrl_bound else { return crash("TypeError: '<=' not supported between instances of 'int' and 'NoneType'") };
    let within = sec_max <= b;
    report.check("AC1", within, format!("the {} section at its maxima is {sec_max} bytes, no more than the control bound (C-17) of {ctrl_bound:?} bytes", t.key));
    counts(t, &root, report);
    let stated = stated_sizes(&t.text);
    report.check("AC1", stated.contains(&sec_max), format!("a sentence of the {} subsection states its maximum section size as {sec_max} bytes (the table's sum); stated {stated:?}", t.key));
    t.code = type_code(&t.text);
    let inside = matches!((t.code, block), (Some(c), Some((lo, hi))) if lo <= c && c <= hi);
    report.check("AC1", inside, format!("the {} section type has a type code inside the block of 004-25 ({block:?}) (found {:?})", t.key, t.code));
    t.root = Some(root);
    Ok(())
}

/// The 64-section report: a sentence that says the bound is exceeded and
/// names Open item 1.
fn report64(stamp: &str) -> bool {
    let p = r"(?i)(64[^.]{0,100}(exceed|more than|above|over)|(exceed|more than|above|over)[^.]{0,100}64)[^.]*Open item 1|Open item 1[^.]*(64[^.]{0,100}(exceed|more than|above|over)|(exceed|more than|above|over)[^.]{0,100}64)";
    rx(p).is_match(stamp)
}

/// AC1 for the seven types; returns them (the ones found, in spec order)
/// and `_S`, the sections they need at their maxima.
pub(crate) fn section_types(st: &Stamp, ctrl_bound: Option<i128>, block: Option<(i128, i128)>, section_max: Option<i128>, report: &mut Report) -> Flow<(Vec<SType>, i128)> {
    let mut found = Vec::new();
    for (key, pat) in TYPE_SPECS {
        let hits = find_sub(&st.subs, pat);
        report.check("AC1", hits.len() == 1, format!("exactly one subsection of the regime stamp section names the {key} section type (found {})", hits.len()));
        if hits.len() == 1 {
            found.push(SType {
                key,
                pat,
                title: hits[0].0.clone(),
                text: hits[0].1.clone(),
                ..SType::default()
            });
        }
    }
    report.check("AC1", found.len() == 7, format!("seven section types are laid out in the regime stamp section (found {})", found.len()));
    for t in &mut found {
        one_type(t, ctrl_bound, block, report)?;
    }
    let codes: Vec<i128> = found.iter().filter_map(|t| t.code).collect();
    let mut distinct = codes.clone();
    distinct.sort_unstable();
    distinct.dedup();
    report.check("AC1", found.len() == 7 && codes.len() == distinct.len() && codes.len() == found.len(), format!("the seven section type codes are distinct (found {codes:?})"));
    let bad = bad_arithmetic(&st.text)?;
    report.check("AC1", bad.is_empty(), format!("every product and sum stated in the regime stamp section is exact: {bad:?}"));
    let s: i128 = found.iter().filter_map(|t| t.nt).filter(|&n| n != 0).sum();
    let bound = section_max.filter(|&m| m != 0).unwrap_or(64);
    report.check("AC1", found.len() == 7 && (s <= bound || report64(&st.text)), format!("the seven types need {s} sections at their maxima; that fits the 64-section bound, or the text reports the excess and names Open item 1"));
    Ok((found, s))
}
