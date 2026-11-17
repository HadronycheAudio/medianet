//! AC1: every hello content has a row, the bundle element and the port
//! element (script lines 527-638).

use crate::doccheck::wp7::md::{Field, fields_of, find_sub, first_table, is_field_table};
use super::{Ctx, Layout, py};
use crate::doccheck::Report;

/// What AC1 finds for the later criteria (row indexes into the layout).
#[derive(Debug, Default)]
pub(crate) struct Rows {
    pub port_row: Option<usize>,
    pub bun_row: Option<usize>,
    pub bun_count: Option<usize>,
    pub has_bundles: bool,
    pub bun_fields: Vec<Field>,
    pub e_bytes: i128,
    pub t_ct: Option<i128>,
}

/// `row_for(pattern, counted)`: the first layout row whose Meaning starts
/// with `pattern`.
pub(crate) fn row_for(lay: &[Field], pattern: &str, counted: Option<bool>) -> Option<usize> {
    lay.iter().position(|f| {
        py::matches(&format!(r"\s*{pattern}"), &f.meaning, true).is_some() && counted.is_none_or(|c| f.maxc.is_some() == c)
    })
}

fn need_row(lay: &[Field], name: &str, pattern: &str, bits: Option<i128>, report: &mut Report) -> Option<usize> {
    let Some(i) = row_for(lay, pattern, None) else {
        report.fail("AC1", format!("the hello layout has no row for {name} (its Meaning must begin with the name)"));
        return None;
    };
    if let Some(b) = bits
        && lay[i].bits != b
    {
        report.fail("AC1", format!("{name} is {} bits; the constants fix {b} bits", lay[i].bits));
    }
    Some(i)
}

fn contents(ctx: &Ctx, lay: &[Field], rows: &mut Rows, report: &mut Report) {
    let k = &ctx.k;
    need_row(lay, "module id (C-11)", r"module\s+id", k.module_bits, report);
    need_row(lay, "instance nonce (C-11)", r"instance\s+nonce", k.nonce_bits, report);
    need_row(lay, "protocol version", r"(protocol\s+)?version", Some(16), report);
    need_row(lay, "name length", r"name\s+length", Some(8), report);
    match row_for(lay, r"name\b", Some(true)) {
        None => report.fail("AC1", "the hello layout has no counted name row (its Meaning must begin with 'Name')"),
        Some(i) => {
            if !py::has("UTF-8|ASCII", &format!("{} {}", lay[i].meaning, lay[i].valid)) {
                report.fail("AC1", "the name row does not state its encoding (UTF-8 or ASCII)");
            }
        }
    }
    if let Some(i) = need_row(lay, "capability bits", "capabilit", None, report) {
        let f = &lay[i];
        if f.bits != 32 {
            report.fail("AC1", format!("capability bits are {} bits; the row must state its bit count (32)", f.bits));
        }
        if !py::has_i(r"holder[- ]eligible[^.;]*?\bbit\s+\d+", &format!("{} {}", f.meaning, f.valid)) {
            report.fail("AC1", "the capability row does not fix the holder-eligible bit at a stated bit (for example 'bit 0')");
        }
    }
    need_row(lay, "port count", r"port\s+count", Some(8), report);
    rows.port_row = row_for(lay, r"ports?\b", Some(true));
    if rows.port_row.is_none() {
        report.fail("AC1", "the hello layout has no counted port row (its Meaning must begin with 'Port')");
    }
    need_row(lay, "election epoch", r"election\s+epoch", None, report);
    need_row(lay, "believed holder (module id, 64 bits)", r"believed\s+holder", k.module_bits, report);
    need_row(lay, "timeline epoch (C-10)", r"timeline\s+epoch", k.epoch_bits, report);
    need_row(lay, "transport role", r"transport\s+role", None, report);
    need_row(lay, "alarm field", "alarm", None, report);
    need_row(lay, "regime stamp version number (C-18)", r"regime\s+stamp", None, report);
    rows.bun_count = row_for(lay, r"bundle\s+count", None);
    for (i, f) in lay.iter().enumerate() {
        if py::matches(r"\s*bundle\b", &f.meaning, true).is_some() && py::matches(r"\s*bundle\s+count", &f.meaning, true).is_none() {
            rows.bun_row = Some(i);
        }
    }
    rows.has_bundles = lay.iter().any(|f| py::matches(r"\s*bundle", &f.meaning, true).is_some());
    let live_in_header = lay.iter().take(7).any(|f| py::has_i(r"liveness\s+counter", &f.meaning));
    if !lay.is_empty() && !live_in_header && ctx.para_with(&[r"message\s+counter", r"liveness\s+counter"]).is_none() {
        report.fail("AC1", "the message counter is not stated as the data-plane liveness counter (C-115)");
    }
    if ctx.para_with(&[r"liveness\s+counter", r"message\s+counter|counter"]).is_none() {
        report.fail("AC1", "no sentence says that the message counter doubles as the liveness counter");
    }
}

fn bundle_element(ctx: &Ctx, rows: &mut Rows, report: &mut Report) {
    let subs = find_sub(&ctx.subs, r"bundle\s+element");
    match subs.first() {
        None => report.fail("AC1", "no '### Bundle element' subsection describing a bundle's group id, member list and channel map"),
        Some((_, x)) => {
            rows.bun_fields = fields_of(first_table(x, is_field_table).as_ref(), "AC2", "bundle element", report);
            let text_b = rows.bun_fields.iter().map(|f| f.meaning.as_str()).collect::<Vec<_>>().join(" ").to_lowercase();
            for want in ["group id", "member", "channel map"] {
                if !text_b.contains(want) {
                    report.fail("AC1", format!("the bundle element has no row for '{want}'"));
                }
            }
        }
    }
    let fields = &rows.bun_fields;
    rows.e_bytes = if fields.is_empty() { 0 } else { fields.iter().map(Field::size_bits).fold(0, i128::saturating_add).div_euclid(8) };
    let mut bo: i128 = 0;
    for f in fields {
        if f.off != bo {
            report.fail("AC2", format!("bundle element: '{}' at byte {}, expected {bo}", py::head(&f.meaning, 30), f.off));
        }
        bo = f.off.saturating_add(f.size_bits().div_euclid(8));
    }
    let Some(mm) = ctx.k.members_max else { return };
    let counted: Vec<&Field> = fields.iter().filter(|f| f.maxc.is_some()).collect();
    for f in &counted {
        let maxc = f.maxc.unwrap_or_default();
        if maxc > mm {
            report.fail("AC2", format!("bundle element row '{}' allows {maxc} members, above C-08's {mm}", py::head(&f.meaning, 40)));
        }
        if py::has_i("member|stream|channel", &f.meaning) && maxc != mm {
            report.fail("AC2", format!("bundle element row '{}' has max {maxc}; C-08 fixes {mm} members", py::head(&f.meaning, 40)));
        }
    }
    if !counted.iter().any(|f| f.maxc == Some(mm)) {
        report.fail("AC2", format!("the bundle element has no counted member row with the maximum of {mm} members (C-08)"));
    }
}

fn port_element(ctx: &Ctx, lay: &[Field], rows: &mut Rows, report: &mut Report) {
    let subs = find_sub(&ctx.subs, r"port\s+element");
    let Some((_, x)) = subs.first() else {
        report.fail("AC1", "no '### Port element' subsection describing a port and its content types");
        return;
    };
    let fields = fields_of(first_table(x, is_field_table).as_ref(), "AC1", "port element", report);
    let ct: Vec<&Field> = fields.iter().filter(|f| py::has_i(r"content\s+type", &f.meaning) && f.maxc.is_some_and(|m| m != 0)).collect();
    match ct.first() {
        None => report.fail("AC1", "the port element has no counted content-type row"),
        Some(f) => rows.t_ct = f.maxc,
    }
    if let Some(p) = rows.port_row
        && !fields.is_empty()
    {
        let pe = fields.iter().map(Field::size_bits).fold(0, i128::saturating_add);
        if lay[p].bits != pe {
            report.fail("AC1", format!("the port row is {} bits per port; its element sums to {pe} bits", lay[p].bits));
        }
    }
}

/// AC1, and the rows AC2 and AC6 read.
pub(crate) fn rows(ctx: &Ctx, lay: &Layout, report: &mut Report) -> Rows {
    let mut rows = Rows::default();
    contents(ctx, &lay.lay, &mut rows, report);
    bundle_element(ctx, &mut rows, report);
    port_element(ctx, &lay.lay, &mut rows, report);
    rows
}
