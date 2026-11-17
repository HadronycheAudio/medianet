//! Port of `checks/004-06.sh`: the media packet header layout (ticket
//! 004-06). The criterion map, with the ticket sentences, is on the tests in
//! `tests/doc_004_06.rs`.
//!
//! The script's variants, reproduced here and shared by the WP5 ports
//! (004-06, 004-08, 004-09, 004-10), all private to this module until the
//! library takes them: Python's text semantics ([`py`]: universal newlines,
//! `splitlines`, `\w`/`\s`/`\d` at Unicode 13, `re.I` folding of `İ` and
//! `ı`, unbounded `int`, `float`, `repr`), the scripts' own TOML subset
//! reader (`wp5::pytoml`), and their Markdown helpers and every-block tag rule
//! ([`md`]). The section is the first heading (any level, outside fences)
//! whose title contains `media\s+packet\s+header` (`re.I`), up to the next
//! heading of the same or a higher level.
//!
//! **Header and code disagree** on the enumeration: the header says "the
//! closed 13-name enumeration"; the code starts from the 13 names and
//! replaces them with the names found in the Rejection reasons tables when
//! at least 10 are found (004-06:650-653, two identical branches), so a
//! section listing only 10 of them shrinks the set the forms are checked
//! against. The port follows the code ([`members`]).
//!
//! The repository-level assertions (`wire-check` reports "checks clean";
//! `wire_tables` 12 passed, 0 failed, 0 ignored; the design checker prints
//! OK; `checks/004-01.sh` passes) are in `scripts/repo-state.d/004-06.sh`.

// The WP5 shared module, compiled into each of the four ports (no port
// imports another, lead ruling R3) until the library takes it.
#[allow(dead_code, clippy::duplicate_mod)]
#[path = "../wp5/mod.rs"]
mod wp5;
mod denials;
mod dfdod;
mod forms;
mod header;
mod toml_side;

use super::{CheckSpec, DiffClass, KnownDifference, Target};
use crate::doccheck::pycompat::splitlines;
use crate::doccheck::{Docs, Report};
use wp5::big::Big;
use wp5::md::{heading_index, is_field_table, is_placeholder, parse_int_token, parse_tables, section_of, strip_fences, width_to_bits};
use wp5::py::{read_universal, shadow, si};

/// The specification.
pub const WIRE_MD: &str = "doc/design/spec/wire.md";
/// The machine-readable tables.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// The harness description of this port.
pub const SPEC: CheckSpec = CheckSpec {
    id: "004-06",
    run,
    script_env: &[("WIRE_MD", WIRE_MD), ("WIRE_TOML", WIRE_TOML)],
    targets: &[
        Target { file: WIRE_MD, heading: Some("^Media packet header$") },
        Target { file: WIRE_MD, heading: Some("^Rejection reasons$") },
        Target { file: WIRE_TOML, heading: None },
    ],
    keywords: &["outside", "must", "reserved", "carries", "sender"],
    known_differences: KNOWN,
    denials: DENIALS,
    toml: crate::doccheck::policy::TomlReader::SubsetReader,
};

use denials::DENIALS;

/// The differences the harness found and a reviewer accepted.
const KNOWN: &[KnownDifference] = &[KnownDifference {
    label: r"_crashes$",
    class: DiffClass::ScriptCrashed,
    reason: "the script raises on these inputs (TypeError: a list as a rejection reason is unhashable in its set test; ValueError: a bad \\u escape in its TOML reader) and prints no FAIL line; the port reports the same input as a CRASH failure",
}];


/// One row of the header's field table.
#[derive(Debug, Clone)]
pub(crate) struct FieldRow {
    /// The cells joined with `" | "`.
    pub text: String,
    /// Byte offset.
    pub off: Big,
    pub bits: Big,
    /// Stated bit positions (lo, hi).
    pub stated: Option<(i64, i64)>,
}

/// wire.md as the script holds it.
pub(crate) struct Ctx {
    pub lines: Vec<String>,
    pub heads: Vec<(usize, usize, String)>,
    /// `sec_i is not None`.
    pub found: bool,
    pub placeholder: bool,
    pub section: String,
    pub prose: String,
    pub field_rows: Vec<FieldRow>,
    pub field_table_found: bool,
    pub field_table_start: Option<usize>,
    pub members: Vec<String>,
}

impl Ctx {
    /// Found and not the placeholder.
    pub fn open(&self) -> bool {
        self.found && !self.placeholder
    }
}


/// The first field table with at least 4 rows whose every row has an offset
/// and a width.
fn field_table(section: &str) -> Option<(usize, Vec<FieldRow>)> {
    for (tstart, header, rows) in parse_tables(section) {
        if !is_field_table(&header) || rows.len() < 4 {
            continue;
        }
        let mut candidate = Vec::new();
        for row in &rows {
            let text = row.join(" | ");
            let (Some(off), Some(bits)) = (parse_int_token(&row[0]), row.get(1).and_then(|c| width_to_bits(c))) else {
                candidate.clear();
                break;
            };
            let stated = wp5::md::bit_positions(&text);
            candidate.push(FieldRow { text, off, bits, stated });
        }
        if !candidate.is_empty() {
            return Some((tstart, candidate));
        }
    }
    None
}


fn load(wire: &str, report: &mut Report) -> Ctx {
    let lines_owned: Vec<String> = splitlines(&shadow(wire)).into_iter().map(str::to_string).collect();
    let lines: Vec<&str> = lines_owned.iter().map(String::as_str).collect();
    let heads = heading_index(&lines);
    let sec = heads.iter().position(|h| si(r"media\s+packet\s+header", &h.2));
    let mut ctx = Ctx {
        members: wp5::members::members(&lines, &heads),
        found: sec.is_some(),
        placeholder: sec.is_none(),
        section: String::new(),
        prose: String::new(),
        field_rows: Vec::new(),
        field_table_found: false,
        field_table_start: None,
        lines: Vec::new(),
        heads: Vec::new(),
    };
    match sec {
        None => {
            for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "DF"] {
                report.fail(ac, "no 'Media packet header' section heading in wire.md");
            }
            report.fail("AC6", "no 'Media packet header' section in wire.md to agree with wire.toml");
        }
        Some(k) => {
            ctx.section = section_of(&lines, &heads, k);
            ctx.prose = strip_fences(&ctx.section);
            if is_placeholder(&ctx.section) {
                ctx.placeholder = true;
                for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "DF"] {
                    report.fail(ac, "the Media packet header section is still the 004-06 placeholder");
                }
            }
        }
    }
    if let Some((start, rows)) = field_table(&ctx.section) {
        ctx.field_rows = rows;
        ctx.field_table_found = true;
        ctx.field_table_start = Some(start);
    }
    if ctx.open() && !ctx.field_table_found {
        for ac in ["AC1", "AC3"] {
            report.fail(ac, "no field table (Offset/Width/Type/Meaning/Valid range) in the Media packet header section");
        }
    }
    ctx.heads = heads;
    ctx.lines = lines_owned;
    ctx
}

/// The check.
pub fn run(docs: &Docs) -> Report {
    let mut report = Report::default();
    let Some(wire) = read_universal(docs, WIRE_MD) else {
        for ac in ["AC1", "AC2", "AC3", "AC4", "AC5", "AC6"] {
            report.fail(ac, format!("{WIRE_MD} does not exist"));
        }
        report.fail("DF", "cannot check — wire.md does not exist");
        report.fail("DoD", "cannot check — wire.md does not exist");
        return report;
    };
    let ctx = load(&wire, &mut report);
    header::ac1(&ctx, &mut report);
    header::ac2(&ctx, &mut report);
    let size = header::ac3(&ctx, &mut report);
    forms::ac4(&ctx, &mut report);
    header::ac5(&ctx, &mut report);
    if let Err(crash) = toml_side::check(docs, &ctx, size.as_ref(), &mut report) {
        let mut crashed = Report::default();
        crashed.fail("CRASH", format!("the script raised an exception reading wire.toml: {crash}"));
        return crashed;
    }
    dfdod::df(&ctx, &mut report);
    dfdod::dod(&ctx, &mut report);
    report
}
