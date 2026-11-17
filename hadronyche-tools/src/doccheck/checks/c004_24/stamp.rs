//! The regime stamp section: its two message tables (AC1) and the numbers
//! computed from them (AC2).

use super::Stop;
use super::landed::Landed;
use super::py::{has_number, nums_before, sentences_lower, tables};
use crate::doccheck::Report;
use crate::doccheck::checks::c004_27::{Ctx, Field, fields_of, is_field_table, subsections};
use crate::doccheck::tables::Table;
use crate::doccheck::text::{has, has_i, ints, rx};
use crate::doccheck::units::{Block, BlockKind, prose_blocks};

/// The section as the criteria read it.
pub(crate) struct Stamp {
    pub text: String,
    pub tables: Vec<Table>,
    pub prose: Vec<Block>,
    /// `SENT`: the lowered sentences of the prose blocks.
    pub sent: Vec<String>,
    pub subs: Vec<(String, String)>,
    pub sec_fl: Vec<Field>,
    pub req_fl: Vec<Field>,
    pub sec_tile: i64,
    pub req_tile: i64,
}

impl Stamp {
    /// `any_sent(*pats)`: one sentence matches every pattern.
    pub fn any_sent(&self, pats: &[&str]) -> bool {
        self.sent.iter().any(|s| pats.iter().all(|p| has(p, s)))
    }

    /// The prose blocks (`k == "p"`).
    pub fn paras(&self) -> impl Iterator<Item = &str> {
        self.prose.iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text.as_str())
    }

    fn stated_max_size(&self, n: i64) -> bool {
        let re = rx(&format!(r"(?:^|\D){n}\s*bytes"));
        self.sent.iter().any(|s| has("maximum|at most|largest|fixed size", s) && re.is_match(s))
    }
}

fn rows_matching<'a>(fl: &'a [Field], pat: &str, minoff: i64) -> Vec<&'a Field> {
    fl.iter().filter(|f| f.off >= minoff && has_i(pat, &f.meaning)).collect()
}

/// `tile_problems`: (bytes, problems).
pub(crate) fn tile_problems(fl: &[Field]) -> (i64, Vec<String>) {
    let (mut cur, mut probs) = (0, Vec::new());
    for f in fl {
        let m: String = f.meaning.chars().take(30).collect();
        if f.off != cur {
            probs.push(format!("'{m}' starts at byte {}, expected {cur}", f.off));
        }
        if f.size_bits() % 8 != 0 {
            probs.push(format!("'{m}' is not a whole number of bytes"));
        }
        cur = f.off + f.size_bits() / 8;
    }
    (cur, probs)
}

fn sig(fl: &[Field]) -> Vec<(i64, i64, String)> {
    fl.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect()
}

fn pick_tables(tabs: &[Table], report: &mut Report) -> (Option<Vec<Field>>, Option<Vec<Field>>) {
    let (mut sec, mut req) = (None, None);
    for t in tabs.iter().filter(|t| is_field_table(t.head())) {
        let fl = fields_of(Some(t), "AC1", "a regime stamp field table", report);
        if sec.is_none() && fl.iter().any(|f| has_i(r"section\W+(index|number)", &f.meaning)) {
            sec = Some(fl);
        } else if req.is_none() && fl.iter().any(|f| has_i(r"module\s+id", &f.meaning)) {
            req = Some(fl);
        }
    }
    (sec, req)
}

fn section_table(st: &Stamp, landed: &Landed, report: &mut Report) -> Result<(), Stop> {
    let fl = &st.sec_fl;
    report.check("AC1", fl.len() >= 8 && sig(fl) == landed.hdr_sig, "the section table opens with the seven common control header rows of the hello, unchanged (Hello, Common control header)");
    let ver: Vec<&Field> = rows_matching(fl, "version", 32)
        .into_iter()
        .filter(|f| !has_i(r"section\W+(count|index|type|length)|number of sections", &f.meaning))
        .collect();
    let ok = ver.len() == 1 && landed.ver_bits.is_some_and(|b| ver[0].bits == b) && ver[0].maxc.is_none_or(|m| m == 0);
    report.check("AC1", ok, format!("the section table carries one stamp version number of the hello's width, {:?} bits (Hello, Regime stamp version number)", landed.ver_bits));
    report.check("AC1", st.any_sent(&["version", r"serial|half the range|modulo|2\^31|hello"]), "the stamp states the version comparison of the hello (serial comparison within one instance nonce)");
    let rows = [
        ("section index", r"section\W+(index|number)"),
        ("section count", r"section\W+count"),
        ("section type", r"section\W+type"),
        ("section length", r"section\W+length"),
        ("typed body", r"\bbody\b"),
    ];
    for (label, pat) in rows {
        report.check("AC1", !rows_matching(fl, pat, 32).is_empty(), format!("the section table carries a row for the {label} (Work item 1)"));
    }
    let (tile, probs) = tile_problems(fl);
    if probs.is_empty() && 0 < tile && landed.ctrl_bound.is_none() {
        return Err(Stop("TypeError: int <= None (C-17 unread)".into()));
    }
    let within = landed.ctrl_bound.is_some_and(|b| 0 < tile && tile <= b);
    report.check("AC1", probs.is_empty() && within, format!("the section table tiles its bytes with no gap and totals {tile} bytes, at most the control bound {:?} (C-17, C-18): {probs:?}", landed.ctrl_bound));
    report.check("AC1", st.stated_max_size(tile), format!("a sentence states the maximum size of a section as {tile} bytes (the table's sum)"));
    Ok(())
}

fn request_table(st: &Stamp, landed: &Landed, report: &mut Report) -> Result<(), Stop> {
    let fl = &st.req_fl;
    report.check("AC1", fl.len() >= 8 && sig(fl) == landed.hdr_sig, "the stamp request opens with the seven common control header rows of the hello, unchanged");
    let module: Vec<&Field> = fl.iter().filter(|f| has_i(r"module\s+id", &f.meaning)).collect();
    let ok = module.len() == 1 && landed.module_bits.is_some_and(|b| module[0].bits == b) && module[0].maxc.is_none_or(|m| m == 0);
    report.check("AC3", ok, format!("the stamp request names the requested module by its {:?}-bit module id (C-11, the hello's width)", landed.module_bits));
    let (tile, probs) = tile_problems(fl);
    if probs.is_empty() && 0 < tile && landed.ctrl_bound.is_none() {
        return Err(Stop("TypeError: int <= None (C-17 unread)".into()));
    }
    let within = landed.ctrl_bound.is_some_and(|b| 0 < tile && tile <= b);
    report.check("AC1", probs.is_empty() && within, format!("the request table tiles its bytes and totals {tile} bytes, at most the control bound (C-17): {probs:?}"));
    report.check("AC1", st.stated_max_size(tile), format!("a sentence states the maximum size of the request as {tile} bytes (the table's sum)"));
    for (label, fl) in [("section", &st.sec_fl), ("request", &st.req_fl)] {
        let codes = fl.iter().find(|f| has_i(r"message\s+type", &f.meaning)).map(|f| ints(&f.valid)).unwrap_or_default();
        let ok = landed.block.is_some() && !codes.is_empty() && codes.iter().all(|c| landed.in_block(*c));
        report.check("AC1", ok, format!("the {label} message type is a code inside the block the hello gives 004-24 (Hello, Control message type codes)"));
    }
    Ok(())
}

/// Reads the regime stamp section and checks its tables (AC1).
pub(crate) fn stamp(ctx: &Ctx, landed: &Landed, report: &mut Report) -> Result<Stamp, Stop> {
    let span = ctx.span_of(r"^regime\s+stamp$", 2);
    let text = ctx.body(span);
    report.check("AC1", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a '## Regime stamp' section with its own text, not the placeholder");
    let tabs = tables(&text);
    let prose = prose_blocks(&text);
    let joined: Vec<&str> = prose.iter().filter(|b| b.kind == BlockKind::Prose).map(|b| b.text.as_str()).collect();
    let sent = sentences_lower(&joined.join("\n"), ".;:");
    let subs = subsections(&text);
    let (sec, req) = pick_tables(&tabs, report);
    if sec.is_none() {
        report.fail("AC1", "the regime stamp has no section table: no field table with a 'section index' row");
    }
    let req_missing = req.is_none();
    let mut st = Stamp {
        text,
        tables: tabs,
        prose,
        sent,
        subs,
        sec_fl: sec.unwrap_or_default(),
        req_fl: req.unwrap_or_default(),
        sec_tile: 0,
        req_tile: 0,
    };
    st.sec_tile = tile_problems(&st.sec_fl).0;
    st.req_tile = tile_problems(&st.req_fl).0;
    section_table(&st, landed, report)?;
    if req_missing {
        report.fail("AC3", "the regime stamp has no request table: no field table with a 'module id' row");
    }
    request_table(&st, landed, report)?;
    Ok(st)
}

fn count_and_constants(ctx: &Ctx, st: &Stamp, report: &mut Report) -> (Option<i64>, Option<i64>, Option<i64>) {
    let cnt = rows_matching(&st.sec_fl, r"section\W+count", 0);
    let m = if cnt.len() == 1 { ints(&cnt[0].valid).last().copied() } else { None };
    report.check("AC2", m.is_some_and(|m| m >= 1), format!("the section count row states a maximum section count in its valid range (found {m:?})"));
    let stated = m.is_some_and(|m| st.sent.iter().any(|s| s.contains("section") && has("maximum|at most|fixed|up to", s) && has_number(s, m)));
    report.check("AC2", stated, format!("a sentence states the maximum section count {m:?} as a number"));
    let grab = |id: &str, pat: &str| rx(pat).captures(ctx.consts.value(id)).and_then(|c| c[1].parse::<i64>().ok());
    let peers = grab("C-217", r"(\d+) peers");
    let modules = grab("C-34", r"at most (\d+)");
    let ok = peers.is_some_and(|p| p != 0) && modules == Some(peers.unwrap_or(0) + 1);
    report.check("AC2", ok, format!("the constants give {peers:?} peers (C-217) and {modules:?} modules (C-34)"));
    (m, peers, modules)
}

type Product = (i128, i128, i128);

fn products(s: &str) -> Vec<Product> {
    rx(r"(\d+)\s*×\s*(\d+)\s*=\s*(\d+)")
        .captures_iter(s)
        .filter_map(|c| Some((c[1].parse().ok()?, c[2].parse().ok()?, c[3].parse().ok()?)))
        .collect()
}

/// AC2: the numbers, computed from the table and the constants.
pub(crate) fn ac2(ctx: &Ctx, st: &Stamp, landed: &Landed, report: &mut Report) -> Result<(), Stop> {
    let (m, peers, modules) = count_and_constants(ctx, st, report);
    let expect = m.filter(|&m| m != 0).map(|m| m * st.sec_tile);
    let stamp_nums: Vec<i64> = st
        .sent
        .iter()
        .filter(|s| s.contains("stamp") && !s.contains("storage") && has("maximum|at most|largest", s))
        .flat_map(|s| nums_before(s, ".", r"\s*bytes"))
        .collect();
    if expect.is_some_and(|e| e != 0 && stamp_nums.contains(&e)) && landed.ctrl_bound.is_none() {
        return Err(Stop("TypeError: int * None (C-17 unread)".into()));
    }
    let ok = expect.is_some_and(|e| e != 0 && stamp_nums.contains(&e) && landed.ctrl_bound.is_some_and(|b| e <= m.unwrap_or(0) * b));
    report.check("AC2", ok, format!("a sentence states the maximum stamp size as {m:?} x {} = {expect:?} bytes (the section count times the section maximum computed from the table, at most the count times the control bound {:?})", st.sec_tile, landed.ctrl_bound));
    let storage: Vec<&String> = st.sent.iter().filter(|s| s.contains("storage")).collect();
    let per: Vec<i64> = storage.iter().flat_map(|s| nums_before(s, ".", r"\s*bytes")).collect();
    let s_max = expect.unwrap_or(0);
    let ok = !per.is_empty() && s_max != 0 && per.iter().all(|c| c % s_max == 0 && *c >= s_max);
    report.check("AC2", ok, format!("every storage figure stated in bytes is a whole multiple, at least one, of the stamp size {s_max} (found {per:?})"));
    let bad: Vec<String> = st.sent.iter().flat_map(|s| products(s)).filter(|(a, b, c)| a * b != *c).map(|(a, b, c)| format!("{a} x {b} = {c}")).collect();
    report.check("AC2", bad.is_empty(), format!("every product stated in the section is exact: {bad:?}"));
    let counts = [peers, modules];
    let bad_totals: Vec<String> = storage
        .iter()
        .flat_map(|s| products(s).into_iter().map(move |p| (p, s)))
        .filter(|((a, b, _), s)| {
            let a_ok = counts.iter().any(|n| n.is_some_and(|n| i128::from(n) == *a));
            let b_ok = per.iter().any(|p| i128::from(*p) == *b);
            !(a_ok && b_ok && has(r"c-217|c-34|entry for itself|does not hold", s))
        })
        .map(|((a, b, c), _)| format!("{a} x {b} = {c}"))
        .collect();
    report.check("AC2", bad_totals.is_empty(), format!("every total over the peers is the per-peer storage times the peer count (C-217 or C-34), with its reason stated: {bad_totals:?}"));
    Ok(())
}
