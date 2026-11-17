//! AC1: the probe section and the two message tables (header rows, tiling,
//! maximum size, type code, padding, identifiers).

use std::collections::BTreeSet;

use super::landed::Landed;
use crate::doccheck::family004::{Ctx, Field, fields_of, first_field_table, subsections};
use crate::doccheck::Report;
use crate::doccheck::pycompat::parse_int;
use crate::doccheck::text::{first_int, has_i, rx};

/// The probe section as the later criteria read it.
pub(crate) struct Probe {
    pub found: bool,
    pub text: String,
    pub subs: Vec<(String, String)>,
    pub req_text: String,
    pub rep_text: String,
    pub req_fl: Vec<Field>,
    pub rep_fl: Vec<Field>,
}

fn is_msg(title: &str) -> bool {
    let t = title.to_lowercase();
    t.contains("probe") && (t.contains("request") || t.contains("reply")) && !rx(r"match|validity|identif|binding").is_match(&t)
}

fn tile_problems(fl: &[Field]) -> (i128, Vec<String>) {
    let (mut cur, mut probs) = (0, Vec::new());
    for f in fl {
        let m: String = f.meaning.chars().take(30).collect();
        if f.off != cur {
            probs.push(format!("'{m}' starts at byte {}, expected {cur}", f.off));
        }
        if f.size_bits() % 8 != 0 {
            probs.push(format!("'{m}' is not a whole number of bytes"));
        }
        cur = f.off.saturating_add(f.size_bits() / 8);
    }
    (cur, probs)
}

/// The stated maximum size; a number beyond i128 equals no row sum, as
/// Python's big int would not.
fn max_size(text: &str) -> Option<i128> {
    rx(r"(?i)maximum\s+size[^.]*?(\d+)\s*bytes").captures(text).and_then(|c| parse_int(&c[1]).ok())
}

/// The per-message problems of AC1.
#[derive(Default)]
struct Problems {
    hdr: Vec<String>,
    tile: Vec<String>,
    size: Vec<String>,
    code: Vec<String>,
    pad: Vec<String>,
    id: Vec<String>,
    codes: BTreeSet<i128>,
}

fn request_rows(fl: &[Field], landed: &Landed, p: &mut Problems) {
    let zero_pad = fl.iter().any(|f| has_i("pad", &format!("{} {}", f.meaning, f.ty)) && has_i(r"\b0\b|zero", &f.valid));
    if !zero_pad {
        p.pad.push("the request has no zero-valued padding row".into());
    }
    if !fl.iter().any(|f| has_i(r"probe\s+identifier|request\s+id", &f.meaning)) {
        p.id.push("request has no probe identifier or request id row".into());
    }
    let width = landed.req_intended.first().map(|f| f.bits);
    let intended = fl.iter().any(|f| rx(r"(?i)^\s*intended\s+packet\s+size").is_match(&f.meaning) && Some(f.bits) == width);
    if !intended {
        p.id.push("request has no intended packet size row at the admission request's width".into());
    }
}

fn reply_rows(fl: &[Field], landed: &Landed, p: &mut Problems) {
    if fl.iter().any(|f| has_i("pad", &format!("{} {}", f.meaning, f.ty))) {
        p.pad.push("the reply has a padding row".into());
    }
    if !fl.iter().any(|f| has_i(r"request\s+id", &f.meaning)) {
        p.id.push("reply has no request id row".into());
    }
    let width = landed.req_intended.first().map(|f| f.bits);
    let echo = fl.iter().any(|f| has_i("echo", &f.meaning) && has_i("size", &f.meaning) && Some(f.bits) == width);
    if !echo {
        p.id.push("reply has no echoed size row at the request's intended packet size width".into());
    }
}

fn one_message(kind: &str, fl: &[Field], text: &str, landed: &Landed, p: &mut Problems) {
    let sig: Vec<(i128, i128, String)> = fl.iter().take(7).map(|f| (f.off, f.bits, f.ty.clone())).collect();
    if fl.len() < 7 || sig != landed.hdr_sig {
        p.hdr.push(kind.to_string());
    }
    let (total, probs) = tile_problems(fl);
    p.tile.extend(probs.into_iter().map(|x| format!("{kind}: {x}")));
    if max_size(text) != Some(total) {
        p.size.push(format!("{kind}: states {:?}, rows sum to {total}", max_size(text)));
    }
    let tf = fl.iter().find(|f| rx(r"(?i)^\s*message\s+type").is_match(&f.meaning));
    // a code beyond i128 is in no block, as Python's big int would not be
    let code = tf.and_then(|f| first_int(&f.valid).ok().flatten());
    if landed.in_block(code) {
        p.codes.insert(code.unwrap_or_default());
    } else {
        p.code.push(format!("{kind}: message type {code:?} is outside the probe block"));
    }
    if kind == "request" {
        request_rows(fl, landed, p);
    } else {
        reply_rows(fl, landed, p);
    }
}

fn table_checks(probe: &Probe, landed: &Landed, report: &mut Report) {
    let mut p = Problems::default();
    one_message("request", &probe.req_fl, &probe.req_text, landed, &mut p);
    one_message("reply", &probe.rep_fl, &probe.rep_text, landed, &mut p);
    let both = !probe.req_fl.is_empty() && !probe.rep_fl.is_empty();
    report.check("AC1", both && p.hdr.is_empty(), format!("each probe message starts with the seven common control header rows in the hello's order, offset, width and type (failing: {:?})", p.hdr));
    report.check("AC1", both && p.tile.is_empty(), format!("each probe message's rows tile from byte 0 with no gap and no overlap: {:?}", p.tile));
    report.check("AC1", both && p.size.is_empty(), format!("each probe message states a maximum size equal to the sum of its rows at their maximum count: {:?}", p.size));
    let rep_ok = matches!((max_size(&probe.rep_text), landed.bound), (Some(s), Some(b)) if s <= b);
    report.check("AC1", rep_ok, format!("the probe reply states a size of at most the {:?}-byte control bound (C-17)", landed.bound));
    report.check("AC1", both && p.code.is_empty() && p.codes.len() == 2, format!("each probe message's Message type is a code inside the block the hello gives the probe pair, and the two differ: {:?}", p.code));
    let fixed = !probe.rep_fl.iter().any(|f| f.maxc.is_some_and(|m| m != 0));
    report.check("AC1", both && p.pad.is_empty() && fixed, format!("the probe request has a zero-valued padding row; the probe reply has no padding row and its rows are of fixed size (no counted row): {:?}", p.pad));
    report.check("AC1", both && p.id.is_empty(), format!("the request carries the intended packet size and an identifier of the probe; the reply carries the request id and echoes the intended packet size: {:?}", p.id));
}

/// Reads the probe section and checks its message tables (AC1).
pub(crate) fn probe(ctx: &Ctx, landed: &Landed, report: &mut Report) -> Probe {
    let span = ctx.span_of(r"^path[-\s]*mtu\s+probe\s+pair$", 2);
    let text = ctx.body(span);
    report.check("AC1", span.is_some() && !text.to_lowercase().contains("to be filled"), "wire.md has a '## Path-MTU probe pair' section with its own text, not the placeholder");
    let subs = subsections(&text);
    let msgs: Vec<&(String, String)> = subs.iter().filter(|(t, _)| is_msg(t)).collect();
    let pick = |w: &str| -> Vec<String> { msgs.iter().filter(|(t, _)| t.to_lowercase().contains(w)).map(|(_, x)| x.clone()).collect() };
    let (req, rep) = (pick("request"), pick("reply"));
    report.check("AC1", req.len() == 1 && rep.len() == 1, format!("the section has one probe request and one probe reply, each a '###' subsection (found {})", msgs.len()));
    let req_text = if req.len() == 1 { req[0].clone() } else { String::new() };
    let rep_text = if rep.len() == 1 { rep[0].clone() } else { String::new() };
    let fl = |t: &str, place: &str, report: &mut Report| {
        if t.is_empty() { Vec::new() } else { fields_of(first_field_table(t).as_ref(), "AC1", place, report) }
    };
    let req_fl = fl(&req_text, "the probe request table", report);
    let rep_fl = fl(&rep_text, "the probe reply table", report);
    let probe = Probe {
        found: span.is_some(),
        text,
        subs,
        req_text,
        rep_text,
        req_fl,
        rep_fl,
    };
    table_checks(&probe, landed, report);
    probe
}
