//! Port of `checks/004-09.sh`: the in-band announcement layouts (ticket
//! 004-09). The criterion map, with the ticket sentences, is on the tests in
//! `tests/doc_004_09.rs`.
//!
//! It uses the WP5 shared module (`checks/wp5`, compiled in with `#[path]`). Its own variants: the
//! section is the first heading whose whole title is `announcements?`
//! (`re.fullmatch`, `re.I`); the validity units are the rows of EVERY table
//! of the section, field tables included (004-09:663-670), and the bullet
//! lines; the reason-token scan also reads every prose block (tables
//! included); small integers are `(?<![\w.\-])(\d{1,3})(?![\w.\-])`
//! ([`small_ints`]); the DEFECT detector of the 004-09 amendment
//! ([`defect`]) reports under `defect` and `defect R6` as the script printed
//! them (the harness reads `FAIL defect R6:` as no criterion: its label
//! pattern stops at the space).
//!
//! **Header and code disagree** on the enumeration: the header says "the
//! closed 13-name enumeration"; the code replaces the 13 names with those
//! found in the Rejection reasons tables when at least 10 are found
//! (004-09:645-646). The port follows the code.
//!
//! The repository-level assertions (`wire-check` reports "checks clean";
//! `wire_tables` stays 12 passed, 0 failed, 0 ignored; the design checker
//! prints OK) are in `scripts/repo-state.d/004-09.sh`. The script does not
//! run checks/004-01.sh.

// The WP5 shared module, compiled into each of the four ports (no port
// imports another, lead ruling R3) until the library takes it.
#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../wp5/mod.rs"]
mod wp5;
mod ac1;
mod ac2;
mod ac345;
mod defect;
mod denials;
mod toml_side;

use wp5::md::{MdTable, blocks, heading_index, is_field_table, is_placeholder, parse_tables, section_of, strip_fences, untagged_blocks};
use wp5::py::{fold_i, head, py_strip, read_universal, shadow};
use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::text::rx;
use crate::doccheck::{Docs, Report};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-09",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Announcements$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_TOML, heading: None },
    ],
    keywords: &["valid", "first", "sender", "future", "skip", "lead"],
    known_differences: KNOWN,
    denials: DENIALS,
    toml: crate::doccheck::policy::TomlReader::SubsetReader,
};

use denials::DENIALS;

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"_crashes$",
    class: DiffClass::ScriptCrashed,
    reason: "the script raises on these inputs (a number where its TOML reader expects a table, a list as a rejection reason) and prints no FAIL line; the port reports the same input as a CRASH failure",
},
    KnownDifference {
        label: r"defect_r6_",
        class: DiffClass::ScriptCrashed,
        reason: "not a difference of verdict: the script printed `FAIL defect R6: …` lines and exited 1, but the harness's criterion pattern `[A-Za-z]+ ?[0-9]*` does not read the label `defect R6`, so a run whose only failures are R6 lines counts as a crash (no FAIL line read); the port reports them under `defect R6`",
    },
    KnownDifference {
        label: r"ac4_c4[34]_not_a_sender_rule$",
        class: DiffClass::PortStricter,
        reason: "not a difference of verdict: the script printed `FAIL defect R6: …` lines and exited 1, but the harness's criterion pattern `[A-Za-z]+ ?[0-9]*` does not read the label `defect R6`, so the script's `defect R6` failure on this input (removing \"sender\" also removes the R6 stop-rule sentence's anchor) is invisible to the harness; the port reports it under `defect R6`",
    },
    KnownDifference {
        label: r"delete (?:section|unit) .*Sender obligations",
        class: DiffClass::PortStricter,
        reason: "not a difference of verdict: deleting the Sender obligations text makes the script print `FAIL defect R6: … does not state the stop rule` (checked by hand on this mutation) and exit 1, but the harness's criterion pattern `[A-Za-z]+ ?[0-9]*` does not read the label `defect R6`; the port reports the same failure under `defect R6`",
    },
];

/// A field table: (its rows as text, its rows, the 5 lines before it).
pub(crate) type FieldTable = (String, Vec<Vec<String>>, String);

/// wire.md as the script holds it.
pub(crate) struct Ctx {
    pub open: bool,
    pub section: String,
    pub prose: String,
    /// `blocks(section_prose)`, tables included.
    pub blks: Vec<String>,
    pub tables: Vec<MdTable>,
    pub field_tables: Vec<FieldTable>,
    /// Every table row and list item, as flat text.
    pub units: Vec<String>,
    pub members: Vec<String>,
}

/// The `(?<![\w.\-])(\d{1,3})(?![\w.\-])` integers of `text` (shadowed).
pub(crate) fn small_ints(text: &str) -> Vec<u32> {
    let edge = |c: Option<char>| c.is_some_and(|c| c == '.' || c == '-' || rx(r"^\w$").is_match(c.encode_utf8(&mut [0u8; 4])));
    rx(r"\d+")
        .find_iter(text)
        .filter(|m| m.as_str().chars().count() <= 3)
        .filter(|m| !edge(text[..m.start()].chars().next_back()) && !edge(text[m.end()..].chars().next()))
        .map(|m| wp5::py::py_int(m.as_str()).and_then(|b| b.to_i64()).expect("three digits") as u32)
        .collect()
}

/// `re.split(r"(?<=[.;])\s+", b)`.
pub(crate) fn split_sentences(b: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for m in rx(r"[.;]\s+").find_iter(b) {
        out.push(b[start..m.start() + 1].to_string());
        start = m.end();
    }
    out.push(b[start..].to_string());
    out
}

/// The prose blocks that do not start with `|` (after `lstrip`).
pub(crate) fn is_table_block(b: &str) -> bool {
    b.trim_start_matches(wp5::py::py_space).starts_with('|')
}

/// `prose_sentences(block_pattern)`: sentences (stripped, `İ`/`ı` folded) of
/// the prose blocks matching `pattern` (`re.I`), tables skipped.
pub(crate) fn prose_sentences(ctx: &Ctx, pattern: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    for b in ctx.blks.iter().filter(|b| !is_table_block(b)) {
        if let Some(p) = pattern
            && !rx(&format!("(?i){p}")).is_match(&fold_i(b))
        {
            continue;
        }
        out.extend(split_sentences(b).iter().map(|s| fold_i(py_strip(s)).into_owned()));
    }
    out
}

fn load(wire: &str, report: &mut Report) -> Ctx {
    let owned: Vec<String> = splitlines(&shadow(wire)).into_iter().map(str::to_string).collect();
    let lines: Vec<&str> = owned.iter().map(String::as_str).collect();
    let heads = heading_index(&lines);
    let sec = heads.iter().position(|h| rx(r"(?i)^(?:announcements?)$").is_match(&fold_i(&h.2)));
    let mut section = String::new();
    let mut open = false;
    let all = ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6", "DoD"];
    match sec {
        None => all.iter().for_each(|ac| report.fail(ac, "no 'Announcements' section heading in wire.md")),
        Some(k) => {
            section = section_of(&lines, &heads, k);
            open = !is_placeholder(&section);
            if !open {
                all.iter().for_each(|ac| report.fail(ac, "the Announcements section is still the 004-09 placeholder"));
            }
        }
    }
    let prose = strip_fences(&section);
    let tables = parse_tables(&section);
    let sec_lines = splitlines(&section);
    let field_tables = tables
        .iter()
        .filter(|t| is_field_table(&t.1))
        .map(|(start, _, rows)| {
            let intro = sec_lines[start.saturating_sub(5)..*start].join("\n");
            (rows.iter().map(|r| r.join(" | ")).collect::<Vec<_>>().join("\n"), rows.clone(), intro)
        })
        .collect();
    let mut units: Vec<String> = tables.iter().flat_map(|t| t.2.iter().map(|r| r.join(" | "))).collect();
    for line in splitlines(&prose) {
        let st = py_strip(line);
        if rx(r"^[-*]\s+").is_match(st) || rx(r"^\d+[.)]\s+").is_match(st) {
            units.push(st.to_string());
        }
    }
    Ctx { open, blks: blocks(&prose), prose, tables, field_tables, units, members: wp5::members::members(&lines, &heads), section }
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let Some(wire) = read_universal(docs, WIRE_MD) else {
        for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6"] {
            report.fail(ac, format!("{WIRE_MD} does not exist"));
        }
        report.fail("DoD", "cannot check — wire.md does not exist");
        return report;
    };
    let ctx = load(&wire, &mut report);
    let container = ac1::ac1(&ctx, &mut report);
    let max_count = ac2::ac2(&ctx, container.as_deref(), &mut report);
    ac345::ac3(&ctx, &mut report);
    ac345::ac4(&ctx, &mut report);
    ac345::ac5(&ctx, &mut report);
    let outcome = toml_side::ac6(docs, &ctx, max_count.as_ref(), &mut report).map(|fields| defect::defect(&ctx, &fields, &mut report));
    if let Err(crash) = outcome {
        let mut crashed = Report::default();
        crashed.fail("CRASH", format!("the script raised an exception: {crash}"));
        return crashed;
    }
    if ctx.open {
        for first in untagged_blocks(&ctx.section) {
            let more = if first.chars().count() > 90 { " …" } else { "" };
            report.fail(
                "DoD",
                format!("this block of the Announcements section carries no evidence tag ([code] [measured] [kernel-doc] [standard] [memory] [decision]): {}{more}", head(&first, 90)),
            );
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_ints_and_sentences_as_python() {
        assert_eq!(small_ints("type 2, 1234, 1.5, -3, a7, 255 x 999"), vec![2, 255, 999]);
        assert_eq!(split_sentences("A b. C; d\ne. F"), vec!["A b.", "C;", "d\ne.", "F"]);
    }
}
