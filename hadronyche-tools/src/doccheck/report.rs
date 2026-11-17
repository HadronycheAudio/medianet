//! The result of a check: every failed assertion with its criterion, its
//! location and its message; and the test helpers that print them all
//! before failing.

use std::collections::BTreeSet;
use std::fmt;

/// Where a failure points: a repository-relative file and a 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loc {
    pub file: String,
    pub line: usize,
}

/// One failed assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The criterion id of the ticket: `AC1`, `AC4`, `DoD`.
    pub criterion: String,
    pub location: Option<Loc>,
    pub message: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(loc) => write!(
                f,
                "FAIL {} {}:{}: {}",
                self.criterion, loc.file, loc.line, self.message
            ),
            None => write!(f, "FAIL {}: {}", self.criterion, self.message),
        }
    }
}

/// Everything a check found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub failures: Vec<Failure>,
    /// The assertions that held, as (criterion, message), for the log.
    pub passes: Vec<(String, String)>,
    /// Set when the script, on these documents, would have stopped with an
    /// uncaught exception (a traceback, no FAIL line, exit 1): the port
    /// models the crash as an outcome of its own, so the harness can match
    /// it to the script's crash (lead ruling R14). A crashed report is red.
    pub crash: Option<String>,
}

impl Report {
    /// Records a failure with no location.
    pub fn fail(&mut self, criterion: &str, message: impl Into<String>) {
        self.failures.push(Failure {
            criterion: criterion.to_string(),
            location: None,
            message: message.into(),
        });
    }

    /// Records a failure at `file:line` (1-based).
    pub fn fail_at(
        &mut self,
        criterion: &str,
        file: &str,
        line: usize,
        message: impl Into<String>,
    ) {
        self.failures.push(Failure {
            criterion: criterion.to_string(),
            location: Some(Loc {
                file: file.to_string(),
                line,
            }),
            message: message.into(),
        });
    }

    /// Records `ok` as a pass or a failure of `criterion`; returns `ok`.
    pub fn check(&mut self, criterion: &str, ok: bool, message: impl Into<String>) -> bool {
        let message = message.into();
        if ok {
            self.passes.push((criterion.to_string(), message));
        } else {
            self.fail(criterion, message);
        }
        ok
    }

    /// True when nothing failed.
    pub fn passed(&self) -> bool {
        self.failures.is_empty() && self.crash.is_none()
    }

    /// Records that the script would crash here (the first crash wins: the
    /// script stops at it).
    pub fn crash(&mut self, why: impl Into<String>) {
        if self.crash.is_none() {
            self.crash = Some(why.into());
        }
    }

    /// The criteria with at least one failure.
    pub fn failed_criteria(&self) -> BTreeSet<String> {
        self.failures.iter().map(|f| f.criterion.clone()).collect()
    }

    /// The failures of one criterion.
    pub fn of(&self, criterion: &str) -> Vec<&Failure> {
        self.failures
            .iter()
            .filter(|f| f.criterion == criterion)
            .collect()
    }

    /// Every failure, one per line.
    pub fn render(&self) -> String {
        let mut lines: Vec<String> = self.failures.iter().map(ToString::to_string).collect();
        if let Some(why) = &self.crash {
            lines.push(format!(
                "CRASH (the script stops with an uncaught exception): {why}"
            ));
        }
        lines.join("\n")
    }
}

/// Panics, printing every failure of `criterion`, when it has any.
#[track_caller]
pub fn assert_criterion(report: &Report, criterion: &str) {
    let found = report.of(criterion);
    if !found.is_empty() {
        let lines: Vec<String> = found.iter().map(ToString::to_string).collect();
        panic!(
            "{} assertion(s) of {criterion} failed:\n{}",
            found.len(),
            lines.join("\n")
        );
    }
}

/// Panics, printing every failure, when the report has any.
#[track_caller]
pub fn assert_clean(report: &Report) {
    if !report.passed() {
        panic!(
            "{} assertion(s) failed:\n{}",
            report.failures.len(),
            report.render()
        );
    }
}

/// Panics unless `criterion` has a failure whose message contains `needle`
/// (for denial fixtures: the mutation must be caught, and by the right
/// assertion). Prints the whole report when it does not.
#[track_caller]
pub fn assert_fails(report: &Report, criterion: &str, needle: &str) {
    let hit = report
        .of(criterion)
        .iter()
        .any(|f| f.message.contains(needle));
    if !hit {
        let all = if report.passed() {
            "(the report has no failure)".to_string()
        } else {
            report.render()
        };
        panic!("expected a failure of {criterion} containing {needle:?}; the report:\n{all}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Report {
        let mut r = Report::default();
        r.check("AC1", true, "fine");
        r.fail("AC1", "first");
        r.fail_at("DoD", "doc/x.md", 12, "second");
        r
    }

    #[test]
    fn failures_carry_criterion_and_location() {
        let r = sample();
        assert!(!r.passed());
        assert_eq!(r.failed_criteria().len(), 2);
        assert_eq!(r.passes.len(), 1);
        assert_eq!(r.render(), "FAIL AC1: first\nFAIL DoD doc/x.md:12: second");
    }

    #[test]
    fn helpers_pass_and_fail_as_documented() {
        let r = sample();
        assert_criterion(&r, "AC2");
        assert_fails(&r, "DoD", "sec");
        let caught = std::panic::catch_unwind(|| assert_criterion(&sample(), "AC1"));
        assert!(caught.is_err());
        let caught = std::panic::catch_unwind(|| assert_fails(&sample(), "AC1", "nope"));
        assert!(caught.is_err());
        assert_clean(&Report::default());
    }
}
