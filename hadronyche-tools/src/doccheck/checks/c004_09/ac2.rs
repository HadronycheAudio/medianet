//! AC2 of 004-09: the maximum number of announcements and of announcement
//! bytes per packet, as numbers, inside the C-137 remainder and consistent
//! with the container field table.

use super::Ctx;
use crate::doccheck::Report;
use super::wp5::big::Big;
use super::wp5::md::width_to_bits;
use super::wp5::py::{find_i, fold_i, py_int, py_strip, s, si};
use crate::doccheck::text::rx;

/// The first match of `pattern` (`re.I`, every match tried in order) whose
/// first non-empty group is above 0.
fn first_positive(pattern: &str, text: &str) -> Option<Big> {
    let folded = fold_i(text);
    rx(&format!("(?i){pattern}")).captures_iter(&folded).find_map(|c| {
        let g = c.iter().skip(1).flatten().map(|m| m.as_str()).find(|x| !x.is_empty())?;
        let v = py_int(g).expect("digits");
        (v > Big::default()).then_some(v)
    })
}

/// The share `pattern` states, or 0.
fn share(pattern: &str, text: &str) -> Big {
    find_i(pattern, text).map_or(Big::default(), |(_, _, g)| py_int(g[0].as_deref().unwrap_or("")).expect("digits"))
}

/// Per-announcement bytes from the container rows, when derivable.
fn per_announcement(rows: &[Vec<String>]) -> Option<Big> {
    let mut fixed = Big::default();
    let mut payload = Big::default();
    for row in rows {
        let joined = row.join(" | ");
        if si(r"\bcount\b|how\s+many", &joined) {
            continue;
        }
        let bits = row.get(1).and_then(|c| width_to_bits(c));
        let ty = py_strip(row.get(2).map_or("", String::as_str)).to_lowercase();
        let fixed_type = rx(r"^(?:u\d+|i\d+|uint|f32|q1\.31|pad)$").is_match(&ty);
        let bits = bits?;
        if si("payload", &joined) && !fixed_type {
            let cell = row.get(4).unwrap_or(&joined);
            if let Some((_, _, g)) = find_i(r"(\d+)\s*bytes?", cell) {
                payload = payload.max(py_int(g[0].as_deref().unwrap_or("")).expect("digits"));
            } else {
                let c = rx(r"0\s+to\s+(\d+)").captures(cell)?;
                if bits != Big::from(8) {
                    return None;
                }
                payload = payload.max(py_int(&c[1]).expect("digits"));
            }
        } else {
            fixed = fixed.add(&bits);
        }
    }
    Some(fixed.divmod_small(8).0.add(&payload))
}

/// AC2; returns the stated maximum count.
pub(crate) fn ac2(ctx: &Ctx, container: Option<&[Vec<String>]>, report: &mut Report) -> Option<Big> {
    if !ctx.open {
        return None;
    }
    let p = &ctx.prose;
    let max_count = first_positive(r"at\s+most\s+(\d+)\s+announcements?|maximum\s+(?:of\s+)?(\d+)\s+announcements?|(\d+)\s+announcements?\s+per\s+packet", p);
    if max_count.is_none() {
        report.fail("AC2", "the maximum NUMBER of announcements per packet is not stated as a number (needed by 004-14)");
    }
    let max_bytes = first_positive(
        r"(\d+)\s+announcement\s+bytes?|announcement\s+bytes[^.;\n]{0,40}?(\d+)|at\s+most\s+(\d+)\s*bytes?[^.;\n]{0,30}?announcements?\s+per\s+packet|(\d+)\s*bytes?\s+of\s+announcements?\b",
        p,
    );
    if max_bytes.is_none() {
        report.fail("AC2", "the maximum ANNOUNCEMENT BYTES per packet is not stated as a number (needed by 004-14)");
    }
    if !s(r"\b62\b", p) && !(s(r"\b100\b", p) && s(r"\b38\b", p)) {
        report.fail("AC2", "the section does not show the C-137 cap arithmetic (100-byte cap, 38 bytes used by the packet + slice headers, 62 remaining for announcements + measurement extension + checksum)");
    }
    let ext = share(r"measurement\s+extension[^.;\n]{0,100}?(\d+)\s*bytes?", p);
    let cks = share(r"checksum[^.;\n]{0,100}?(\d+)\s*bytes?", p);
    let available = Big::from(62).sub(&ext.add(&cks));
    if let Some(mb) = &max_bytes
        && *mb > available
    {
        report.fail("AC2", format!("the stated maximum announcement bytes ({mb}) do not fit the C-137 remainder: 62 bytes minus the stated shares (measurement extension {ext}, checksum {cks}) leave {available}"));
    }
    if let Some(per) = container.and_then(per_announcement)
        && let (Some(mb), Some(mc)) = (&max_bytes, &max_count)
        && mc.mul(&per) > *mb
    {
        report.fail("AC2", format!("the stated numbers are not self-consistent with the field tables: {mc} announcements x {per} bytes (container fields + payload maximum) = {} bytes, above the stated maximum of {mb}", mc.mul(&per)));
    }
    max_count
}
