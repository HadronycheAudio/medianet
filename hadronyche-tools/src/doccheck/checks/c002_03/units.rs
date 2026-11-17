//! AC3 of 002-03: every quoted number on a quantity-bearing line has a
//! unit. The script's awk program (`unit_offenders`) runs on BYTES (mawk):
//! it is ported here statement by statement on `family002::awk`, and the
//! whole function is proven against mawk on the corpus (snippet `002-03
//! unit_offenders`, `tests/doc_002_03.rs`).
//!
//! Faithful to the program, including what it does not mean to do:
//!
//! * the quantity cue is tested on the LOWERCASED record, so its
//!   upper-case alternatives (`_(LEN|NS|…)`, `STATE_LEN`, `ROOM`, …) never
//!   match there, while the line filter before it (`grep -Eqi`) is
//!   case-insensitive;
//! * the operator test takes ONE byte: `×` is written in the bracket as its
//!   two bytes, so a number followed by `×` or by `÷` (same first byte) is
//!   a factor;
//! * the 24 characters after a number are 24 bytes.

use super::Subs;
use crate::doccheck::Report;
use crate::doccheck::family002::awk::{AwkRe, substr, tolower, v_assign};
use crate::doccheck::family002::{Grep, tr_delete_space};

/// The script's `quantity_re` (grep's and, through `-v cue=`, awk's).
pub const QUANTITY_RE: &str = r"(^|[^[:alnum:]_])(constant|const|timeout|period|delay|interval|rate|size|length|budget|window|lead|expiry|expir|election|listen|repeat|room|capacity|duration|deadline|payload|every|per second|bytes|samples?|ticks?|frames?)([^[:alnum:]_]|$)|_(LEN|NS|MS|SECS|BYTES|SLOTS)([^[:alnum:]_]|$)|(^|[^[:alnum:]_])(STATE_LEN|PULSE_LEN|REPORT_LEN|ROOM|CYCLE_SECS|DIVIDER|SLOTS|LISTEN_NS|HELLO_NS|EXPIRE_NS|ELECTION_NS)([^[:alnum:]_]|$)";

/// The script's `unit_re` (awk's, through `-v unit=`).
pub const UNIT_RE: &str = r"(^|[^[:alnum:]_])(ms|milliseconds?|µs|us|microseconds?|ns|nanoseconds?|s|sec|secs|seconds?|minutes?|hours?|Hz|kHz|MHz|bytes?|KiB|KB|bits?|samples?|ticks?|frames?|fps|ppm|%|Q32\.32|Q1\.31|Q0\.32|slots?|entries|channels?|controllers?|slices?|blocks?)([^[:alnum:]_]|$)|per second";

/// The identifiers an assignment exempts unless dimensioned.
const DIMENSIONED: [&str; 5] = ["ROOM", "CYCLE_SECS", "DIVIDER", "SLOTS", "ONE"];

/// `unit_offenders "$line"`: the lines the awk program prints (without
/// their newlines), as bytes.
pub fn unit_offenders(line: &str) -> Vec<Vec<u8>> {
    let record = line.as_bytes();
    if !AwkRe::new(&v_assign(QUANTITY_RE)).test(&tolower(record)) {
        return Vec::new();
    }
    let unit = AwkRe::new(&v_assign(UNIT_RE));
    let mut s = record.to_vec();
    for p in [
        r"[A-Za-z0-9_.\/-]+\.rs:[0-9]+(-[0-9]+)?",
        r"C-[0-9]+",
        r"[0-9][0-9][0-9]-[0-9][0-9]",
        r"Q[0-9]+\.[0-9]+",
        r"[0-9][0-9]?:[0-9][0-9]:[0-9][0-9][:;][0-9][0-9]",
        r"\^[ ]*[0-9]+",
        r"<<[ ]*[0-9]+",
    ] {
        s = AwkRe::new(p).replace(&s, b"", true);
    }
    let mut out = Vec::new();
    while let Some((nstart, nlen)) = AwkRe::new(r"[0-9]+([.,][0-9]+)?").find(&s) {
        let (nstart, nlen) = (nstart as i64, nlen as i64);
        let adv = nstart + nlen;
        let num = substr(&s, nstart, Some(nlen)).to_vec();
        let before = substr(&s, 1, Some(nstart - 1)).to_vec();
        let after = substr(&s, adv, Some(24)).to_vec();
        let rest = substr(&s, adv, None).to_vec();
        if !is_quoted(&before, &after, &unit) {
            s = rest;
            continue;
        }
        let mut msg = b"number ".to_vec();
        msg.extend_from_slice(&num);
        msg.extend_from_slice(b" has no unit near: ");
        msg.extend_from_slice(&after);
        out.push(msg);
        s = rest;
    }
    out
}

/// The program's exemptions, in its order: false when the number is a
/// factor (an operator follows it or precedes it), has a unit within 24
/// bytes, or is assigned to a non-dimensioned upper-case identifier.
fn is_quoted(before: &[u8], after: &[u8], unit: &AwkRe) -> bool {
    let tail = AwkRe::new(r"^[ \t]*").replace(after, b"", false);
    if AwkRe::new(r"[+*\/xX<×-]").test(substr(&tail, 1, Some(1))) {
        return false;
    }
    if unit.test(&tolower(after)) {
        return false;
    }
    let prev = AwkRe::new(r"[ \t]+$").replace(before, b"", false);
    let prevch = substr(&prev, prev.len() as i64, Some(1));
    if AwkRe::new(r"[\^+*\/<]").test(prevch) {
        return false;
    }
    if let Some((start, len)) = AwkRe::new(r"([A-Z][A-Z0-9_]{2,})[`*_]*[ \t]*=[ \t]*$").find(before) {
        let ident = substr(before, start as i64, Some(len as i64));
        let ident = AwkRe::new(r"[`*_]*[ \t]*=[ \t]*$").replace(ident, b"", false);
        let dimensioned = AwkRe::new(r"(_LEN|_NS|_MS|_SECS|_BYTES|_SLOTS)$").test(&ident)
            || DIMENSIONED.iter().any(|d| ident == d.as_bytes());
        if !dimensioned {
            return false;
        }
    }
    true
}

/// AC3, per sub-section: every quantity-bearing line (not blank, not a
/// heading, with a digit and a quantity cue) gives a unit for every quoted
/// number.
pub(crate) fn ac3(s: &Subs, r: &mut Report) {
    let heading = Grep::ere(r"^#+[[:space:]]");
    let digit = Grep::ere("[0-9]");
    let cue = Grep::ere_i(QUANTITY_RE);
    for (label, (start, end)) in [("transport.rs", s.t), ("control.rs", s.c)] {
        let (mut nbad, mut nlines) = (0, 0);
        for k in start..=end {
            let line = s.lines.sed_line(k);
            if tr_delete_space(line).is_empty() || heading.matches(line) {
                continue;
            }
            if !(digit.matches(line) && cue.matches(line)) {
                continue;
            }
            nlines += 1;
            let offenders = unit_offenders(line);
            if !offenders.is_empty() {
                nbad += 1;
                for o in offenders {
                    r.fail("AC3", format!("{label} line {k}: {}", String::from_utf8_lossy(&o)));
                }
            }
        }
        if nbad == 0 {
            r.check(
                "AC3",
                true,
                format!("{label}: all {nlines} quantity-bearing lines give a unit for every quoted number"),
            );
        } else {
            r.fail("AC3", format!("{label}: {nbad} of {nlines} quantity-bearing lines lack a unit for a quoted number"));
        }
    }
}
