//! AC2 of 004-10: the byte size stated as a number and equal to the field
//! widths, the fields contiguous, the C-137 arithmetic (100 − 38 = 62,
//! 62 − 49 = 13) and the checksum remainder or the fallback omission.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::py::{find_i, py_int, s, si};

/// The stated size: the first `extension … N bytes` or `N bytes extension`.
fn stated(prose: &str) -> Option<Big> {
    let (_, _, g) = find_i(
        r"extension[^.;\n|]{0,60}?\b(\d+)\s*-?\s*bytes?\b|\b(\d+)\s*-?\s*bytes?\s+(?:measurement\s+)?extension\b",
        prose,
    )?;
    let digits = g.into_iter().flatten().find(|x| !x.is_empty()).expect("one group");
    Some(py_int(&digits).expect("digits"))
}

/// Sum and contiguity of the three widths.
fn widths(ctx: &Ctx, stated: Option<&Big>, report: &mut Report) {
    let Some(all) = ctx.all() else { return };
    let Some(bits): Option<Vec<Big>> = all.iter().map(|e| e.bits.clone().filter(|b| !b.is_zero())).collect() else {
        return;
    };
    let sum = bits.iter().fold(Big::default(), |a, b| a.add(b)).divmod_small(8).0;
    if let Some(n) = stated
        && *n != sum
    {
        report.fail("AC2", format!("the stated extension size ({n} bytes) does not equal the sum of its field widths ({sum} bytes)"));
    }
    let mut order: Vec<usize> = vec![0, 1, 2];
    order.sort_by_key(|&k| (all[k].off.is_none(), all[k].off.clone().unwrap_or_default()));
    if order.iter().any(|&k| all[k].off.is_none()) {
        report.fail("AC2", "an extension field row states no offset");
        return;
    }
    let base = all[order[0]].off.clone().expect("checked");
    let mut cum = Big::default();
    for &k in &order {
        let off = all[k].off.as_ref().expect("checked");
        if off.sub(&base) != cum {
            let key = super::fields::KEYS[k].0;
            report.fail("AC2", format!("the extension fields are not contiguous: {key} at offset {off} where {cum} bytes of the extension precede it"));
            break;
        }
        cum = cum.add(&bits[k].divmod_small(8).0);
    }
}

/// AC2; returns the stated size.
pub(crate) fn ac2(ctx: &Ctx, report: &mut Report) -> Option<Big> {
    if !ctx.open {
        return None;
    }
    let n = stated(&ctx.prose);
    if n.is_none() {
        report.fail("AC2", "the extension's byte size is not stated as a number");
    }
    widths(ctx, n.as_ref(), report);
    let p = &ctx.prose;
    let nums = ["100", "38", "62", "49", "13"].iter().all(|x| s(&format!(r"\b{x}\b"), p));
    if !nums || !s("004-13", p) {
        report.fail(
            "AC2",
            "the C-137 arithmetic is not shown (100-byte cap, 38 for the headers, 62 remaining, 49 for a maximal announcement container, 13 for the extension and the checksum together, the checksum share from 004-13)",
        );
    }
    if let Some(n) = &n {
        if *n <= Big::from(13) {
            let rem = Big::from(13).sub(n);
            let said = ctx.prose_blks.iter().any(|b| si("checksum", b) && s("004-13", b) && si(&format!(r"\b{rem}\s*bytes?\b"), b));
            if !said {
                report.fail("AC2", format!("the checksum remainder is not stated: 13 - {n} = {rem} bytes are left for the checksum (004-13)"));
            }
            if !si(r"\bfits?\b|\bfitting\b", p) {
                report.fail("AC2", "the section does not say whether the extension fits beside a maximal announcement container");
            }
        } else if !(si("fallback", p) && si("omit", p) && si(r"\bwhen\b|\bif\b", p)) {
            report.fail(
                "AC2",
                format!("the extension ({n} bytes) does not fit beside a maximal announcement container (13 bytes), and no omission on a fallback path is stated, with its condition"),
            );
        }
    }
    n
}
