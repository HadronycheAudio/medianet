//! AC6 (document part): wire.toml agrees with the Markdown table for table;
//! review T2 (the whole manifest above the bound needs a stated reason, the
//! fixed part declared row for row). wire.toml is read with the scripts'
//! subset reader; values are compared as Python compares them (`8 == 8.0`,
//! a missing key is `None`).

use super::MRow;
use super::encoding::code_for;
use super::fields::Man;
use super::landed::Landed;
use super::size::Top;
use super::tile;
use crate::doccheck::Report;
use crate::doccheck::pyvalue::{Dict, PyValue};
use crate::doccheck::report::{Flow, crash};
use crate::doccheck::sentences::{RAW_COLON, split_raw};
use crate::doccheck::text::{has, has_i};
use crate::doccheck::tomlsubset;

/// (bit offset, width, max count) of a row; `None` for no count.
pub(crate) type Sig = Vec<(i128, i128, Option<i128>)>;

/// wire.toml as the later checks read it.
pub(crate) struct Toml {
    /// `TOML_MSGS`, iterated as Python iterates it.
    pub msgs: Vec<PyValue>,
    pub fixed_msgs: Vec<Dict>,
    pub fixed_md: Vec<MRow>,
    pub fixed_offs: Vec<i128>,
}

/// `for x in v`: a list's items, a string's characters, a dict's keys; a
/// `TypeError` otherwise.
fn py_iter(v: Option<&PyValue>) -> Flow<Vec<PyValue>> {
    match v {
        None => Ok(Vec::new()),
        Some(PyValue::List(xs)) => Ok(xs.clone()),
        Some(PyValue::Str(s)) => Ok(s.chars().map(|c| PyValue::Str(c.to_string())).collect()),
        Some(PyValue::Dict(d)) => Ok(d.keys().map(|k| PyValue::Str(k.to_string())).collect()),
        Some(other) => crash(format!("TypeError: '{}' object is not iterable", other.type_name())),
    }
}

/// `m.get("field", []) if isinstance(m, dict) else []`, each element used
/// with `.get`.
fn fields(m: &PyValue) -> Flow<Vec<Dict>> {
    let Some(d) = m.as_dict() else { return Ok(Vec::new()) };
    py_iter(d.get("field"))?
        .into_iter()
        .map(|f| match f {
            PyValue::Dict(f) => Ok(f),
            other => crash(format!("AttributeError: '{}' object has no attribute 'get'", other.type_name())),
        })
        .collect()
}

/// `v == n` with `None == None`.
fn opt_eq(v: Option<&PyValue>, n: Option<i128>) -> bool {
    match (v, n) {
        (None, None) => true,
        (Some(v), Some(n)) => v.eq_int(n),
        _ => false,
    }
}

/// `[(f.get("offset"), f.get("width"), f.get("max_count")) ...] == sig`.
fn sig_eq(fs: &[Dict], sig: &Sig) -> bool {
    fs.len() == sig.len()
        && fs.iter().zip(sig).all(|(f, (o, w, c))| opt_eq(f.get("offset"), Some(*o)) && opt_eq(f.get("width"), Some(*w)) && opt_eq(f.get("max_count"), *c))
}

impl Toml {
    /// The dict messages whose fields have `sig`.
    fn matching(&self, sig: &Sig) -> Flow<Vec<Dict>> {
        let mut out = Vec::new();
        for m in &self.msgs {
            if sig_eq(&fields(m)?, sig)
                && let Some(d) = m.as_dict()
            {
                out.push(d.clone());
            }
        }
        Ok(out)
    }

    /// Point 4: the meaning and valid text of the fields at bit `off` of the
    /// messages with `sig`.
    pub fn texts_at(&self, sig: &Sig, off: i128) -> Flow<Vec<String>> {
        let mut out = Vec::new();
        for m in &self.msgs {
            let fs = fields(m)?;
            if !sig_eq(&fs, sig) {
                continue;
            }
            for f in fs.iter().filter(|f| opt_eq(f.get("offset"), Some(off))) {
                out.push(format!("{} {}", str_of(f, "meaning")?, str_of(f, "valid")?));
            }
        }
        Ok(out)
    }

    /// T3 (b), wire.toml side: the fixed part's E field states the same
    /// code as the Markdown row.
    pub fn sentinel_checks(&self, kind: &str, code: i128, report: &mut Report) -> Flow<()> {
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
            let fs = fields(&PyValue::Dict(first.clone()))?;
            let tf: Vec<&Dict> = fs.iter().filter(|f| opt_eq(f.get("offset"), Some(*off))).collect();
            let tcode = match tf.first() {
                Some(f) => code_for(&str_of(f, "valid")?, kind),
                None => None,
            };
            report.check("AC2", tcode == Some(code), format!("wire.toml's valid range for the {kind} field states {code:#x} as the markdown does (read {tcode:?})"));
        }
        Ok(())
    }
}

/// `f.get(key, "")` used as a string (`+ " "`, `re.search`).
fn str_of(f: &Dict, key: &str) -> Flow<String> {
    match f.get(key) {
        None => Ok(String::new()),
        Some(PyValue::Str(s)) => Ok(s.clone()),
        Some(other) => crash(format!("TypeError: can only concatenate str (not \"{}\") to str", other.type_name())),
    }
}

fn whole_ok(man: &Man) -> bool {
    man.paras_flat.iter().flat_map(|p| split_raw(p, RAW_COLON)).any(|s| {
        has_i("schema|wire-check|checker", s) && has_i("cannot|can not|does not|not expressible|no single|not declare", s) && has(r"\b0\d\d-\d\d\b|\b004-02\b", s)
    })
}

fn good(m: &Dict, size: i128, landed: &Landed) -> Flow<bool> {
    let base = m.get("layer").is_some_and(|v| v.eq_str("control")) && opt_eq(m.get("dscp"), Some(40)) && opt_eq(m.get("max_size"), Some(size));
    if !base {
        return Ok(false);
    }
    // `all(r in REJECT_SET for r in ...)` stops at the first miss
    for r in py_iter(m.get("rejection_reasons"))? {
        let r = match r {
            PyValue::List(_) | PyValue::Dict(_) => return crash(format!("TypeError: unhashable type: '{}'", r.type_name())),
            other => other.py_str(),
        };
        if !landed.reject.contains(&r) {
            return Ok(false);
        }
    }
    Ok(fields(&PyValue::Dict(m.clone()))?.iter().all(|f| f.contains_key("max_count") == f.contains_key("count")))
}

fn tables_agree(man: &Man, landed: &Landed, toml: &Toml, report: &mut Report) -> Flow<()> {
    let whole = whole_ok(man);
    for m in &man.models {
        let size = m.total.div_euclid(8);
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

/// Reads wire.toml (AC6's parse). The script catches the reader's
/// `TomlError` only (an AC6 failure); any other exception of the reader
/// (`ValueError`, `IndexError`) is a crash.
pub(crate) fn parse(text: Option<&str>, report: &mut Report) -> Flow<Toml> {
    let root = match text {
        None => {
            report.fail("AC6", "doc/design/spec/wire.toml does not exist");
            None
        }
        Some(t) => match tomlsubset::parse(t) {
            Ok(r) => Some(r),
            Err(e) if e.starts_with("TomlError") => {
                report.fail("AC6", format!("wire.toml does not parse: {e}"));
                None
            }
            Err(e) => return crash(e),
        },
    };
    let msgs = py_iter(root.as_ref().and_then(|r| r.get("message")))?;
    Ok(Toml { msgs, fixed_msgs: Vec::new(), fixed_md: Vec::new(), fixed_offs: Vec::new() })
}

/// AC6 table for table (T2's first part).
pub(crate) fn models(man: &Man, landed: &Landed, toml: &Toml, report: &mut Report) -> Flow<()> {
    tables_agree(man, landed, toml, report)
}

/// T2's fixed part, declared row for row.
pub(crate) fn fixed(man: &Man, top: Option<&Top>, toml: &mut Toml, report: &mut Report) -> Flow<()> {
    let model = top.map(|t| &man.models[t.model]);
    toml.fixed_md = model.map(|m| m.rows.iter().take_while(|r| r.maxc.is_none()).cloned().collect()).unwrap_or_default();
    toml.fixed_offs = if toml.fixed_md.is_empty() { Vec::new() } else { tile(&toml.fixed_md).0 };
    let fixed_bytes = toml.fixed_md.iter().map(|r| r.bits).fold(0i128, i128::saturating_add).div_euclid(8);
    let sig: Sig = toml.fixed_md.iter().zip(&toml.fixed_offs).map(|(r, o)| (*o, r.bits, None)).collect();
    toml.fixed_msgs = if sig.is_empty() { Vec::new() } else { toml.matching(&sig)? };
    report.check("AC6", !toml.fixed_msgs.is_empty(), format!("wire.toml declares the manifest fixed part as a [[message]] whose rows agree with the markdown fixed part row for row ({} found)", toml.fixed_msgs.len()));
    let ok = toml.fixed_msgs.iter().any(|m| opt_eq(m.get("max_size"), Some(fixed_bytes)) && m.get("layer").is_some_and(|v| v.eq_str("control")) && opt_eq(m.get("dscp"), Some(40)));
    report.check("AC6", ok, format!("the fixed-part message has max_size equal to the markdown fixed-part sum ({fixed_bytes} bytes), layer control, dscp 40"));
    Ok(())
}
