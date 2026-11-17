//! Reading wire.toml as the four WP5 scripts did (their subset reader,
//! [`super::pytoml`]), and the Python container access they share. An
//! exception the script did not catch is returned as `Err` ([`Crash`]).

use super::md::norm_name;
use super::py::read_universal;
use super::pytoml::{self, Dict, Fail, Value};
use crate::doccheck::{Docs, Report};

/// The machine-readable tables.
pub(crate) const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// An exception the script did not catch.
pub(crate) type Crash = String;

/// Reads wire.toml: `Ok(None)` when it is missing or unreadable (reported
/// under `ac`), `Err` on a crash.
pub(crate) fn load(docs: &Docs, ac: &str, report: &mut Report) -> Result<Option<Dict>, Crash> {
    let Some(text) = read_universal(docs, WIRE_TOML) else {
        report.fail(ac, format!("{WIRE_TOML} does not exist"));
        return Ok(None);
    };
    match pytoml::parse(&text) {
        Ok(d) => Ok(Some(d)),
        Err(Fail::Error(e)) => {
            report.fail(ac, format!("{WIRE_TOML} could not be read by this check: {e}"));
            Ok(None)
        }
        Err(Fail::Crash(e)) => Err(e),
    }
}

/// `iter_or_empty`, with a crash as `Err`.
pub(crate) fn items(v: Option<&Value>) -> Result<Vec<Value>, Crash> {
    Value::iter_or_empty(v).map_err(|f| match f {
        Fail::Crash(e) | Fail::Error(e) => e,
    })
}

/// The first `[[message]]` whose `str(name)` normalises to something
/// containing `needle`.
pub(crate) fn message(doc: &Dict, needle: &str) -> Result<Option<Dict>, Crash> {
    for m in items(doc.get("message"))? {
        if let Value::Dict(d) = m {
            let name = d.get("name").map_or(String::new(), Value::py_str);
            if norm_name(&name).contains(needle) {
                return Ok(Some(d));
            }
        }
    }
    Ok(None)
}

/// `repr(x)` of an optional value (`None` when absent).
pub(crate) fn repr(v: Option<&Value>) -> String {
    v.map_or("None".to_string(), Value::repr)
}
