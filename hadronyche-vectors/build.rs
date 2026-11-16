//! Build step of `hadronyche-vectors` (ticket 007a-03, Work 3): embed the
//! vector files in the crate so a device without a file system can iterate
//! them.
//!
//! The build script runs on the host and may use `std`. It walks the embedded
//! roots, parses every `*.json` vector with `serde_json` (the standard parser,
//! not a hand-written one), checks the fields the reader relies on, and writes
//! `$OUT_DIR/vectors.rs`: one static table per kind, whose entries are
//! constant Rust data (strings, numbers, nested slices) and whose wire bytes
//! are `include_bytes!` of the `.bin` sidecar. Nothing is left to parse at run
//! time.
//!
//! Embedded roots:
//!
//! * `vectors/` — always; the crate's conformance data (007a-05 onward).
//! * `tests/fixtures/vectors/examples/` — only with the non-default `examples`
//!   feature: the six hand-written example fixtures of 007a-02, which are
//!   schema samples, not conformance vectors.
//!
//! A malformed file fails the build with one line per problem, naming the
//! file. The full 007a-02 schema (unknown fields, closed sets beyond the
//! rejection reason, counter names) is checked by `tests/vector_files.rs`;
//! this script checks what the reader embeds and exposes.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// The closed `Reject` enumeration of ticket 004-03 (`doc/design/spec/wire.toml`),
/// in value order. Each name is also a variant of `reader::Reason`; the
/// generated code names the variant, so a name missing there fails to compile.
const REJECT_REASONS: [&str; 13] = [
    "TooShort",
    "BadMagic",
    "VersionMismatch",
    "ForeignDomain",
    "UnconfiguredDomain",
    "UnknownStream",
    "UnadmittedSource",
    "BadLength",
    "BadChecksum",
    "BadPacketIndex",
    "WrongEpoch",
    "OversizeControl",
    "ReservedNonZero",
];

/// The six kinds: the generated table's name, the reader's variant, and the
/// directory below an embedded root that fixes the kind (007a-02).
const KINDS: [(&str, &str, &str); 6] = [
    ("WIRE_VALID", "WireValid", "wire/valid"),
    ("WIRE_INVALID", "WireInvalid", "wire/invalid"),
    ("CONTROL", "Control", "control"),
    ("TIMELINE", "Timeline", "timeline"),
    ("TRANSPORT", "Transport", "transport"),
    ("REFERENCE", "Reference", "reference"),
];

/// Index of wire valid in [`KINDS`].
const WIRE_VALID: usize = 0;
/// Index of wire invalid in [`KINDS`].
const WIRE_INVALID: usize = 1;

/// One parsed vector file, ready to be written out.
struct Entry {
    path: PathBuf,
    kind: usize,
    id: String,
    spec_version: String,
    /// The `.bin` sidecar of a wire vector, absolute.
    bin: Option<PathBuf>,
    doc: Map<String, Value>,
}

fn main() {
    let manifest = PathBuf::from(env_var("CARGO_MANIFEST_DIR"));
    let mut roots = vec![manifest.join("vectors")];
    if std::env::var_os("CARGO_FEATURE_EXAMPLES").is_some() {
        roots.push(manifest.join("tests/fixtures/vectors/examples"));
    }
    println!("cargo:rerun-if-changed=build.rs");
    for root in &roots {
        println!("cargo:rerun-if-changed={}", root.display());
    }

    let mut errors = Vec::new();
    let entries = collect(&roots, &mut errors);
    if errors.is_empty() {
        unique_ids(&entries, &mut errors);
    }
    if !errors.is_empty() {
        eprintln!("hadronyche-vectors: malformed vector data; the build embeds none of it:");
        for error in &errors {
            eprintln!("  {error}");
        }
        std::process::exit(1);
    }

    let out = PathBuf::from(env_var("OUT_DIR")).join("vectors.rs");
    if let Err(e) = fs::write(&out, generate(&entries)) {
        eprintln!("hadronyche-vectors: cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
}

fn env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        eprintln!("hadronyche-vectors: cargo did not set {name}");
        std::process::exit(1);
    })
}

/// Parse every `*.json` file below the roots, in root order then path order.
fn collect(roots: &[PathBuf], errors: &mut Vec<String>) -> Vec<Entry> {
    let mut entries = Vec::new();
    for root in roots {
        let mut files = Vec::new();
        if let Err(e) = walk_json(root, &mut files) {
            errors.push(format!("{}: {e}", root.display()));
            continue;
        }
        files.sort();
        for file in files {
            println!("cargo:rerun-if-changed={}", file.display());
            match parse(root, &file) {
                Ok(entry) => entries.push(entry),
                Err(mut e) => errors.append(&mut e),
            }
        }
    }
    entries
}

fn walk_json(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk_json(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "json") {
            out.push(path);
        }
    }
    Ok(())
}

/// The kind a file's path names below its root, as an index into [`KINDS`].
fn kind_of(root: &Path, file: &Path) -> Option<usize> {
    let rel = file.strip_prefix(root).ok()?;
    let rel = rel.to_str()?.replace('\\', "/");
    KINDS
        .iter()
        .position(|(_, _, dir)| rel.starts_with(&format!("{dir}/")))
}

/// Parse one file and check the fields the reader embeds.
fn parse(root: &Path, file: &Path) -> Result<Entry, Vec<String>> {
    let at = file.display();
    let kind = kind_of(root, file).ok_or_else(|| {
        vec![format!(
            "{at}: not below wire/valid, wire/invalid, control, timeline, transport or reference"
        )]
    })?;
    let text = fs::read_to_string(file).map_err(|e| vec![format!("{at}: {e}")])?;
    let doc = match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map,
        Ok(_) => return Err(vec![format!("{at}: a vector file must be a JSON object")]),
        Err(e) => return Err(vec![format!("{at}: not valid JSON: {e}")]),
    };

    let mut errors = Vec::new();
    let id = required_str(&doc, "id", &mut errors);
    let spec_version = required_str(&doc, "spec_version", &mut errors);
    let mut bin = None;
    if kind == WIRE_VALID || kind == WIRE_INVALID {
        required_str(&doc, "message", &mut errors);
        bin = check_bin(file, &doc, &mut errors);
    }
    if kind == WIRE_VALID {
        check_valid(&doc, &mut errors);
    } else if kind == WIRE_INVALID {
        let reason = required_str(&doc, "reason", &mut errors);
        if !reason.is_empty() && !REJECT_REASONS.contains(&reason.as_str()) {
            errors.push(format!(
                "`reason` is {reason:?}, not a member of the closed Reject enumeration \
                 of ticket 004-03 ({})",
                REJECT_REASONS.join(", ")
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors.into_iter().map(|e| format!("{at}: {e}")).collect());
    }
    Ok(Entry {
        path: file.to_path_buf(),
        kind,
        id,
        spec_version,
        bin,
        doc,
    })
}

/// A mandatory non-empty string field; empty when absent (an error is pushed).
fn required_str(doc: &Map<String, Value>, key: &str, errors: &mut Vec<String>) -> String {
    match doc.get(key).and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => s.to_owned(),
        _ => {
            errors.push(format!("`{key}` must be a non-empty string"));
            String::new()
        }
    }
}

/// A wire vector's `.bin` sidecar: it exists, sits beside the JSON file, and
/// its size equals `length_bytes`.
fn check_bin(file: &Path, doc: &Map<String, Value>, errors: &mut Vec<String>) -> Option<PathBuf> {
    let name = required_str(doc, "bin", errors);
    if name.is_empty() {
        return None;
    }
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        errors.push(format!(
            "`bin` is {name:?}: it must name a file beside the JSON file"
        ));
        return None;
    }
    let path = file.parent().unwrap_or(Path::new(".")).join(&name);
    let size = match fs::metadata(&path) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(_) => {
            errors.push(format!("`bin`: {} is not a file", path.display()));
            return None;
        }
        Err(e) => {
            errors.push(format!("`bin`: {}: {e}", path.display()));
            return None;
        }
    };
    match doc.get("length_bytes").and_then(Value::as_u64) {
        Some(n) if n == size => {}
        Some(n) => errors.push(format!(
            "`length_bytes` is {n} but {} is {size} bytes",
            path.display()
        )),
        None => errors.push("`length_bytes` must be a non-negative integer".to_owned()),
    }
    println!("cargo:rerun-if-changed={}", path.display());
    Some(path)
}

/// A wire-valid vector's recorded decoded structure and encoding case.
fn check_valid(doc: &Map<String, Value>, errors: &mut Vec<String>) {
    if !doc.get("decoded").is_some_and(Value::is_object) {
        errors.push("`decoded` must be an object (the decoded structure)".to_owned());
    }
    let case = doc.get("case").and_then(Value::as_str).unwrap_or("");
    if case_variant(case).is_none() {
        errors.push(format!(
            "`case` is {case:?}; it must be \"minimal\", \"typical\" or \"maximal\""
        ));
    }
}

fn case_variant(case: &str) -> Option<&'static str> {
    match case {
        "minimal" => Some("Minimal"),
        "typical" => Some("Typical"),
        "maximal" => Some("Maximal"),
        _ => None,
    }
}

/// Ids are unique across the embedded set (007a-02 Work 2).
fn unique_ids(entries: &[Entry], errors: &mut Vec<String>) {
    let mut seen: BTreeMap<&str, &Path> = BTreeMap::new();
    for entry in entries {
        if let Some(first) = seen.insert(&entry.id, &entry.path) {
            errors.push(format!(
                "duplicate id {:?}: {} and {}",
                entry.id,
                first.display(),
                entry.path.display()
            ));
        }
    }
}

// ------------------------------------------------------------ generation ---

/// The generated module body: one `pub(super) static` slice per kind.
fn generate(entries: &[Entry]) -> String {
    let mut out = String::from("// @generated by hadronyche-vectors/build.rs; do not edit.\n");
    for (index, (table, _, dir)) in KINDS.iter().enumerate() {
        let _ = writeln!(out, "\n/// Embedded `{dir}` vectors.");
        let _ = writeln!(out, "pub(super) static {table}: &[Vector] = &[");
        for entry in entries.iter().filter(|e| e.kind == index) {
            write_entry(&mut out, entry);
        }
        out.push_str("];\n");
    }
    out
}

fn write_entry(out: &mut String, entry: &Entry) {
    let variant = KINDS[entry.kind].1;
    let _ = writeln!(out, "    // {:?}", entry.path);
    let _ = writeln!(out, "    Vector {{");
    let _ = writeln!(out, "        kind: Kind::{variant},");
    let _ = writeln!(out, "        id: {:?},", entry.id);
    let _ = writeln!(out, "        spec_version: {:?},", entry.spec_version);
    match &entry.bin {
        Some(bin) => {
            let path = bin.to_str().unwrap_or_else(|| {
                eprintln!("hadronyche-vectors: {}: path is not UTF-8", bin.display());
                std::process::exit(1);
            });
            let _ = writeln!(out, "        bytes: include_bytes!({path:?}),");
        }
        None => out.push_str("        bytes: &[],\n"),
    }
    let _ = writeln!(out, "        outcome: {},", outcome(entry));
    let _ = writeln!(out, "        document: &{},", fields(&entry.doc));
    out.push_str("    },\n");
}

/// The recorded outcome: the decoded structure, the rejection reason, or
/// `NotWire` for the four scenario kinds.
fn outcome(entry: &Entry) -> String {
    let text = |key: &str| entry.doc.get(key).and_then(Value::as_str).unwrap_or("");
    match entry.kind {
        WIRE_VALID => format!(
            "Outcome::Valid(Decoded {{ message: {:?}, case: Case::{}, value: {} }})",
            text("message"),
            case_variant(text("case")).unwrap_or("Minimal"),
            value(entry.doc.get("decoded").unwrap_or(&Value::Null))
        ),
        WIRE_INVALID => format!(
            "Outcome::Rejected(Rejection {{ reason: Reason::{} }})",
            text("reason")
        ),
        _ => "Outcome::NotWire".to_owned(),
    }
}

/// A JSON value as a constant `reader::Value` expression.
fn value(v: &Value) -> String {
    match v {
        Value::Null => "Value::Null".to_owned(),
        Value::Bool(b) => format!("Value::Bool({b})"),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                format!("Value::Int({i})")
            } else if let Some(u) = n.as_u64() {
                format!("Value::UInt({u})")
            } else {
                // serde_json yields finite floats only; `{:?}` round-trips.
                format!("Value::Float({:?})", n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => format!("Value::Str({s:?})"),
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(value).collect();
            format!("Value::Array(&[{}])", items.join(", "))
        }
        Value::Object(map) => format!("Value::Object(&{})", fields(map)),
    }
}

/// An object's members as a constant `[Field]` array, in key order.
fn fields(map: &Map<String, Value>) -> String {
    let items: Vec<String> = map
        .iter()
        .map(|(name, v)| format!("Field {{ name: {name:?}, value: {} }}", value(v)))
        .collect();
    format!("[{}]", items.join(", "))
}
