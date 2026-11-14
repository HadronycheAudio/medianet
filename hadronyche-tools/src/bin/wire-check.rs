//! `wire-check` — run the ticket 004-02 machine-readable wire-table checker
//! on `doc/design/spec/wire.toml` from the command line.
//!
//! A thin wrapper over [`hadronyche_tools::wire_tables::check_file`] so later
//! tickets can self-check their table additions locally; CI runs the same
//! check through `cargo test` in `hadronyche-tools`.

use std::path::PathBuf;

use hadronyche_tools::wire_tables::check_file;

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("hadronyche-tools is a workspace member at the repository root")
        .join("doc")
        .join("design")
        .join("spec")
        .join("wire.toml");
    match check_file(&path) {
        Ok(report) => {
            println!(
                "{}: {} message(s), checks clean",
                path.display(),
                report.messages.len()
            );
        }
        Err(error) => {
            eprintln!(
                "{}: {:?}: {}",
                path.display(),
                error.kind,
                error.message
            );
            std::process::exit(1);
        }
    }
}
