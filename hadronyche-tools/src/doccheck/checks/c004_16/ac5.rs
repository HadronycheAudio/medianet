//! AC5: the common control header (defined once, its size, its order
//! against the sync header) and the hello layout that starts with it
//! (script lines 417-524).

use super::md::{Field, key_of, fields_of, find_sub, first_table, is_field_table, prose_blocks};
use super::{Ctx, Layout, NUM, py, to_int};
use crate::doccheck::Report;

fn key_label(k: &str) -> &'static str {
    match k {
        "magic" => "magic",
        "version" => "version",
        "domain" => "domain",
        "type" => r"message\s+type",
        "scheme" => "scheme",
        "counter" => "counter",
        _ => r"\btag\b",
    }
}

fn header(ctx: &Ctx, out: &mut Layout, report: &mut Report) {
    let subs = find_sub(&ctx.subs, r"common\s+control\s+header");
    let mut table = None;
    match subs.first() {
        None => report.fail("AC5", "no '### Common control header' subsection in the Hello section"),
        Some((_, x)) => {
            out.hdr_sub = Some(x.clone());
            table = first_table(x, is_field_table);
            if table.is_none() {
                report.fail("AC5", "the common control header subsection has no five-column field table");
            }
        }
    }
    out.hdr = fields_of(table.as_ref(), "AC5", "common control header", report);
    out.hdr_keys = out.hdr.iter().map(|f| key_of(&f.meaning)).collect();
    let mut set: Vec<Option<&str>> = out.hdr_keys.clone();
    set.sort();
    set.dedup();
    let want = ["counter", "domain", "magic", "scheme", "tag", "type", "version"];
    if !out.hdr.is_empty() && set != want.iter().map(|k| Some(*k)).collect::<Vec<_>>() {
        report.fail("AC5", format!("the common control header must carry magic, version, domain, message type, authentication scheme, counter and tag; it carries {:?}", out.hdr_keys));
    }
    let k = &ctx.k;
    for (f, key) in out.hdr.iter().zip(&out.hdr_keys) {
        let want = match key.unwrap_or("") {
            "magic" | "version" => Some(16),
            "type" => Some(8),
            "domain" => k.domain_bits,
            "scheme" => k.scheme_bits,
            "counter" => k.counter_bits,
            "tag" => k.tag_bits,
            _ => None,
        };
        if let Some(w) = want
            && f.bits != w
        {
            report.fail("AC5", format!("common control header {} is {} bits; the constants fix {w} bits", key.unwrap_or(""), f.bits));
        }
    }
    let mut cur: i128 = 0;
    for f in &out.hdr {
        if f.off != cur {
            report.fail("AC5", format!("common control header: {} at byte {}, expected {cur}", py::head(&f.meaning, 30), f.off));
        }
        cur = cur.saturating_add(f.bits.div_euclid(8));
    }
    if !out.hdr.is_empty() && cur != 32 {
        report.fail("AC5", format!("the common control header sums to {cur} bytes (the sync header is 32)"));
    }
    stated_size(ctx, &out.hdr, cur, report);
}

fn stated_size(ctx: &Ctx, hdr: &[Field], total: i128, report: &mut Report) {
    let stated = ctx
        .para_with(&[r"common\s+control\s+header", r"\d"])
        .and_then(|p| py::search(&format!(r"common\s+control\s+header[^\n]*?({NUM})\s*bytes"), p, true))
        .and_then(|m| to_int(m.g(1)));
    match stated {
        None => report.fail("AC5", "no sentence states the common control header's size in bytes"),
        Some(s) if !hdr.is_empty() && s != total => {
            report.fail("AC5", format!("the common control header states {s} bytes; its table sums to {total}"));
        }
        Some(_) => {}
    }
    if ctx.para_with(&["reuse", "004-17", "004-18", "004-24", "004-27"]).is_none() {
        report.fail("AC5", "no sentence says that 004-17, 004-18, 004-24 and 004-27 reuse the common control header");
    }
}

fn sync_order(ctx: &Ctx, out: &Layout, report: &mut Report) {
    let mut sync_keys: Vec<Option<&str>> = Vec::new();
    match ctx.span(r"^sync\s+request\s+and\s+reply", Some(2)) {
        None => report.fail("AC5", "no 'Sync request and reply' section in wire.md to compare the header with"),
        Some(span) => {
            if let Some(st) = first_table(&ctx.text(span), is_field_table) {
                // `key_of(r[3])` raises IndexError on a row of fewer than
                // four cells; the script dies there.
                for r in st.rows.iter().take(7) {
                    match r.get(3) {
                        Some(c) => sync_keys.push(key_of(c)),
                        None => {
                            report.fail("CRASH", "IndexError: a sync header row has fewer than four cells");
                            return;
                        }
                    }
                }
            }
        }
    }
    if sync_keys.is_empty() || out.hdr_keys.is_empty() || sync_keys == out.hdr_keys {
        return;
    }
    let differing: Vec<&str> = out
        .hdr_keys
        .iter()
        .enumerate()
        .filter(|(i, k)| *i >= sync_keys.len() || sync_keys[*i] != **k)
        .map(|(_, k)| k.unwrap_or("None"))
        .collect();
    let excuse = prose_blocks(out.hdr_sub.as_deref().unwrap_or(""))
        .into_iter()
        .find(|(k, t)| *k == 'p' && py::has_i(r"differs\s+from\s+the\s+sync\s+header", t))
        .map(|(_, t)| t);
    let Some(excuse) = excuse else {
        report.fail("AC5", format!("the hello header order {:?} differs from the sync header order {sync_keys:?}; the only excuse is a paragraph in the Common control header subsection that says 'differs from the sync header' and names the fields", out.hdr_keys));
        return;
    };
    for k in differing {
        // `KEY_LABEL[k]` raises KeyError for a row of no known field.
        if k == "None" {
            report.fail("CRASH", "KeyError: None (a header row names no known field)");
            return;
        }
        if !py::has_i(key_label(k), &excuse) {
            report.fail("AC5", format!("the 'differs from the sync header' paragraph does not name the differing field '{k}'"));
        }
    }
}

fn hello_layout(ctx: &Ctx, out: &mut Layout, report: &mut Report) {
    let subs = find_sub(&ctx.subs, "layout");
    let mut table = None;
    match subs.first() {
        None => report.fail("AC5", "no '### Hello layout' subsection with the hello's field table"),
        Some((_, x)) => {
            table = first_table(x, is_field_table);
            if table.is_none() {
                report.fail("AC5", "the hello layout subsection has no five-column field table");
            }
        }
    }
    out.lay = fields_of(table.as_ref(), "AC1", "hello layout", report);
    let lay = &out.lay;
    if !lay.is_empty() {
        let keys: Vec<Option<&str>> = lay.iter().take(7).map(|f| key_of(&f.meaning)).collect();
        if lay.len() < 7 || keys != out.hdr_keys {
            report.fail("AC5", "the hello's own table does not start with the common control header rows");
        } else {
            for (a, b) in lay.iter().take(7).zip(&out.hdr) {
                if (a.off, a.bits, &a.ty) != (b.off, b.bits, &b.ty) {
                    report.fail("AC5", format!("hello header row {} differs from the common control header in offset, width or type", py::head(&b.meaning, 30)));
                }
            }
        }
    }
    let mut cur: i128 = 0;
    for f in lay {
        if f.off != cur {
            report.fail("AC5", format!("hello layout: '{}' at byte {}, expected {cur} (rows must tile)", py::head(&f.meaning, 40), f.off));
        }
        if f.size_bits().rem_euclid(8) != 0 {
            report.fail("AC5", format!("hello layout: '{}' is not a whole number of bytes", py::head(&f.meaning, 40)));
        }
        cur = f.off.saturating_add(f.size_bits().div_euclid(8));
    }
    out.h_bytes = cur;
}

/// AC5 and the layout the later criteria read.
pub(crate) fn layout(ctx: &Ctx, report: &mut Report) -> Layout {
    let mut out = Layout::default();
    header(ctx, &mut out, report);
    sync_order(ctx, &out, report);
    hello_layout(ctx, &mut out, report);
    out
}
