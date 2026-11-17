//! AC6 (document part): wire.toml agrees with the Markdown table for table;
//! review T2 (the whole manifest above the bound needs a stated reason, the
//! fixed part declared row for row). wire.toml is read with the scripts'
//! subset reader; values are compared as Python compares them (`8 == 8.0`,
//! a missing key is `None`).

use toml::{Table, Value};

use super::MRow;
use super::encoding::code_for;
use super::fields::Man;
use super::landed::Landed;
use super::size::Top;
use super::tile;
use crate::doccheck::Report;
use crate::doccheck::checks::c004_24::Stop;
use crate::doccheck::checks::c004_24::py::split_after;
use crate::doccheck::checks::c004_24::pytoml::{eq_int, parse_toml, py_iter};
use crate::doccheck::wire_toml::display;
use crate::doccheck::text::{has, has_i};

/// (bit offset, width, max count) of a row; `None` for no count.
pub(crate) type Sig = Vec<(i64, i64, Option<i64>)>;

/// wire.toml as the later checks read it.
pub(crate) struct Toml {
    /// `TOML_MSGS`.
    pub msgs: Vec<Value>,
    pub fixed_msgs: Vec<Table>,
    pub fixed_md: Vec<MRow>,
    pub fixed_offs: Vec<i64>,
}

/// `m.get("field", [])` of a dict message (`[]` for anything else), each
/// element used with `.get`.
fn fields(m: &Value) -> Result<Vec<Table>, String> {
    let Value::Table(t) = m else { return Ok(Vec::new()) };
    py_iter(t.get("field"))?
        .into_iter()
        .map(|f| match f {
            Value::Table(f) => Ok(f),
            other => Err(format!("AttributeError: {other} has no get")),
        })
        .collect()
}

/// `[(f.get("offset"), f.get("width"), f.get("max_count")) ...] == sig`.
fn sig_eq(fs: &[Table], sig: &Sig) -> bool {
    fs.len() == sig.len()
        && fs.iter().zip(sig).all(|(f, (o, w, c))| eq_int(f.get("offset"), Some(*o)) && eq_int(f.get("width"), Some(*w)) && eq_int(f.get("max_count"), *c))
}

impl Toml {
    /// The dict messages whose fields have `sig`.
    fn matching(&self, sig: &Sig) -> Result<Vec<Table>, String> {
        let mut out = Vec::new();
        for m in &self.msgs {
            if sig_eq(&fields(m)?, sig)
                && let Value::Table(t) = m {
                    out.push(t.clone());
                }
        }
        Ok(out)
    }

    /// Point 4: the meaning and valid text of the fields at bit `off` of the
    /// messages with `sig`.
    pub fn texts_at(&self, sig: &Sig, off: i64) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for m in &self.msgs {
            let fs = fields(m)?;
            if !sig_eq(&fs, sig) {
                continue;
            }
            for f in fs.iter().filter(|f| eq_int(f.get("offset"), Some(off))) {
                out.push(format!("{} {}", str_of(f, "meaning")?, str_of(f, "valid")?));
            }
        }
        Ok(out)
    }

    /// T3 (b), wire.toml side: the fixed part's E field states the same
    /// code as the Markdown row.
    pub fn sentinel_checks(&self, kind: &str, code: i128, report: &mut Report) -> Result<(), String> {
        let Some(first) = self.fixed_msgs.first() else { return Ok(()) };
        if self.fixed_md.is_empty() {
            return Ok(());
        }
        for (r, off) in self.fixed_md.iter().zip(&self.fixed_offs) {
            let m = r.meaning.to_lowercase();
            let wanted = (kind == "none" && m.starts_with("required e")) || (kind == "unknown" && m.starts_with("declared e"));
            if !wanted {
                continue;
            }
            let fs = fields(&Value::Table(first.clone()))?;
            let tf: Vec<&Table> = fs.iter().filter(|f| eq_int(f.get("offset"), Some(*off))).collect();
            let tcode = match tf.first() {
                Some(f) => code_for(&str_of(f, "valid")?, kind),
                None => None,
            };
            report.check("AC2", tcode == Some(code), format!("wire.toml's valid range for the {kind} field states {code:#x} as the markdown does (read {tcode:?})"));
        }
        Ok(())
    }
}

/// `f.get(key, "")` used as a string.
fn str_of(f: &Table, key: &str) -> Result<String, String> {
    match f.get(key) {
        None => Ok(String::new()),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(other) => Err(format!("TypeError: {key} = {other} is not a string")),
    }
}

fn whole_ok(man: &Man) -> bool {
    man.paras_flat.iter().flat_map(|p| split_after(p, ".;:")).any(|s| {
        has_i("schema|wire-check|checker", &s) && has_i("cannot|can not|does not|not expressible|no single|not declare", &s) && has(r"\b0\d\d-\d\d\b|\b004-02\b", &s)
    })
}

fn good(m: &Table, size: i128, landed: &Landed) -> Result<bool, String> {
    let size = i64::try_from(size).unwrap_or(i64::MAX);
    if !(m.get("layer").and_then(Value::as_str) == Some("control") && eq_int(m.get("dscp"), Some(40)) && eq_int(m.get("max_size"), Some(size))) {
        return Ok(false);
    }
    // `all(r in REJECT_SET for r in ...)` stops at the first miss.
    for r in py_iter(m.get("rejection_reasons"))? {
        let r = match r {
            Value::Array(_) | Value::Table(_) => return Err("TypeError: unhashable".into()),
            other => display(&other),
        };
        if !landed.reject.contains_key(&r) {
            return Ok(false);
        }
    }
    Ok(fields(&Value::Table(m.clone()))?.iter().all(|f| f.contains_key("max_count") == f.contains_key("count")))
}

fn tables_agree(man: &Man, landed: &Landed, toml: &Toml, report: &mut Report) -> Result<(), String> {
    let whole = whole_ok(man);
    for m in &man.models {
        let size = i128::from(m.total.div_euclid(8));
        if size > landed.bound {
            report.check("AC6", whole, format!("{} ({size} bytes) is above the control bound: wire.toml cannot declare it as one message, and the section says so with an owner for the tooling gap", m.place));
            continue;
        }
        let sig: Sig = m.rows.iter().zip(&m.offs).map(|(r, o)| (*o, r.bits, r.maxc)).collect();
        let matches = toml.matching(&sig)?;
        report.check("AC6", !matches.is_empty(), format!("{} has a [[message]] in wire.toml with the same fields (offsets, widths, maximum counts) ({} found)", m.place, matches.len()));
        let mut any = false;
        for t in &matches {
            any |= good(t, size, landed)?;
        }
        report.check("AC6", any, format!("{}: a matching toml message is layer control, dscp 40 (C-13), its max_size equals the computed {size} bytes, its reasons are Reject members, and each counted field carries both count and max_count", m.place));
    }
    Ok(())
}

fn fixed_part(man: &Man, top: Option<&Top>, toml: &mut Toml, report: &mut Report) -> Result<(), String> {
    let model = top.map(|t| &man.models[t.model]);
    toml.fixed_md = model.map(|m| m.rows.iter().take_while(|r| r.maxc.is_none()).cloned().collect()).unwrap_or_default();
    toml.fixed_offs = if toml.fixed_md.is_empty() { Vec::new() } else { tile(&toml.fixed_md).0 };
    let fixed_bytes = toml.fixed_md.iter().map(|r| r.bits).sum::<i64>().div_euclid(8);
    let sig: Sig = toml.fixed_md.iter().zip(&toml.fixed_offs).map(|(r, o)| (*o, r.bits, None)).collect();
    toml.fixed_msgs = if sig.is_empty() { Vec::new() } else { toml.matching(&sig)? };
    report.check("AC6", !toml.fixed_msgs.is_empty(), format!("wire.toml declares the manifest fixed part as a [[message]] whose rows agree with the markdown fixed part row for row ({} found)", toml.fixed_msgs.len()));
    let ok = toml.fixed_msgs.iter().any(|m| eq_int(m.get("max_size"), Some(fixed_bytes)) && m.get("layer").and_then(Value::as_str) == Some("control") && eq_int(m.get("dscp"), Some(40)));
    report.check("AC6", ok, format!("the fixed-part message has max_size equal to the markdown fixed-part sum ({fixed_bytes} bytes), layer control, dscp 40"));
    Ok(())
}

/// AC6 (document part) and T2; returns wire.toml for the later checks.
/// Reads wire.toml (AC6's parse). A `TomlError` of the reader is a failure
/// of AC6; any other exception of the reader stops the script (it catches
/// `TomlError` only).
pub(crate) fn parse(text: Option<&str>, report: &mut Report) -> Result<Toml, Stop> {
    let root = match text {
        None => {
            report.fail("AC6", "doc/design/spec/wire.toml does not exist");
            None
        }
        Some(t) => match parse_toml(t) {
            Ok(r) => Some(r),
            Err(e) if ["AttributeError", "IndexError", "ValueError", "TypeError"].iter().any(|p| e.starts_with(p)) => return Err(Stop(e)),
            Err(e) => {
                report.fail("AC6", format!("wire.toml does not parse: {e}"));
                None
            }
        },
    };
    let msgs = py_iter(root.as_ref().and_then(|r| r.get("message"))).map_err(Stop)?;
    Ok(Toml { msgs, fixed_msgs: Vec::new(), fixed_md: Vec::new(), fixed_offs: Vec::new() })
}

/// AC6 table for table (T2's first part).
pub(crate) fn models(man: &Man, landed: &Landed, toml: &Toml, report: &mut Report) -> Result<(), Stop> {
    tables_agree(man, landed, toml, report).map_err(Stop)
}

/// T2's fixed part, declared row for row.
pub(crate) fn fixed(man: &Man, top: Option<&Top>, toml: &mut Toml, report: &mut Report) -> Result<(), Stop> {
    fixed_part(man, top, toml, report).map_err(Stop)
}
