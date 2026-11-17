//! The criterion of a `FAIL: <message>` line: the "colon form" the epic-002
//! scripts print (`bad() { printf 'FAIL: %s\n' "$*"; }`, Python
//! `print('FAIL: ' + msg)`).
//!
//! Those scripts name no criterion of their own: most messages start with
//! one (`AC 2: section (d) ...`, `DoD: ...`, `Hand-offs: ...`), others are
//! free text (`section (e) Run confirmations not found (...)`, `stale
//! sync-rate statement (...) 02-constants.md:12: ...`). The rule, shared by
//! the harness (which reads the script's output) and the ports (which name
//! their failures with it, so the two always agree):
//!
//! * a message that starts with a label — letters, digits, spaces,
//!   hyphens, dots, then `:` — has that label as its criterion, trimmed, the
//!   acceptance-criterion forms normalised as everywhere (`AC 1`, `AC-1`
//!   read `AC1`);
//! * any other message is its own criterion, whole: each such assertion is
//!   compared message for message.
//!
//! A harness run reads the colon form only for a port that asks for it
//! (`DOCCHECK_FAIL_FORM=colon` in its `script_env`), so no other script's
//! summary line (`FAIL: 3 assertion(s) failed`) becomes a criterion.

use super::text::rx;

/// The criterion of the message of a `FAIL: <message>` line.
pub fn colon_criterion(message: &str) -> String {
    let label = rx(r"^([A-Za-z0-9][A-Za-z0-9 .\-]*?)\s*:");
    let Some(c) = label.captures(message) else {
        return message.to_string();
    };
    let name = c[1].to_string();
    match rx(r"^AC[ -]?([0-9]+)$").captures(&name) {
        Some(n) => format!("AC{}", &n[1]),
        None => name,
    }
}

#[cfg(test)]
mod tests {
    use super::colon_criterion;

    #[test]
    fn labels_are_read_and_acceptance_criteria_normalised() {
        assert_eq!(colon_criterion("AC 1: 002-AC-1 needs exactly one"), "AC1");
        assert_eq!(colon_criterion("AC-7: x"), "AC7");
        assert_eq!(colon_criterion("AC12: x: y"), "AC12");
        assert_eq!(colon_criterion("DoD: every paragraph"), "DoD");
        assert_eq!(colon_criterion("Hand-offs: missing x"), "Hand-offs");
        assert_eq!(
            colon_criterion("002-10 open item: closed"),
            "002-10 open item"
        );
        assert_eq!(
            colon_criterion("report missing: doc/x.md"),
            "report missing"
        );
        assert_eq!(colon_criterion("v1.2 rule : q"), "v1.2 rule");
    }

    #[test]
    fn free_text_is_its_own_criterion() {
        for m in [
            "section (e) Run confirmations not found (heading containing '(e)')",
            "stale sync-rate statement (37 or 77) adr/24.md:16: a sync rate of 37",
            "the open item about the POC loss accounting names no owner",
            "no paragraph starting \"Open item\" about it",
            "3 assertion(s) failed",
            "the subsection's CONFIG_HZ=250 does not match: x",
            ": leading colon",
            "",
        ] {
            assert_eq!(colon_criterion(m), m);
        }
    }
}
