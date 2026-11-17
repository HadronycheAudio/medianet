//! The committed proof tables (`data/family002/`), and the one entry point a
//! port's test uses to prove a composite of its own against the real tool:
//! [`check_snippet`]. A script function that combines several tools (002-03's
//! awk `unit_offenders`) is run verbatim by the generator on every corpus
//! input; its outputs are committed as digests; the port's Rust function is
//! compared with them without running any tool.

use std::collections::BTreeSet;

use super::ere::unescape_field;

/// The corpus: inputs, per-pattern answers, snippet digests, fixtures.
pub(crate) const CORPUS: &str = include_str!("../../../data/family002/corpus.tsv");

/// The rows of a table (comment lines skipped), split on tabs.
pub(crate) fn rows(table: &str) -> impl Iterator<Item = Vec<&str>> {
    table
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect())
}

/// An id list `0-3,7` (or `-`).
pub(crate) fn ids(spec: &str) -> Vec<usize> {
    let mut out = BTreeSet::new();
    if spec == "-" {
        return Vec::new();
    }
    for part in spec.split(',') {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        if let (Some(a), Some(b)) = (index(a), index(b)) {
            out.extend(a..=b);
        }
    }
    out.into_iter().collect()
}

/// A decimal index of the tables.
pub(crate) fn index(s: &str) -> Option<usize> {
    crate::doccheck::pycompat::parse_int(s)
        .ok()
        .and_then(|n| usize::try_from(n).ok())
}

/// The corpus inputs, by id.
pub(crate) fn inputs() -> Vec<String> {
    rows(CORPUS)
        .filter(|r| r[0] == "I")
        .map(|r| unescape_field(r.get(2).copied().unwrap_or("")))
        .collect()
}

/// FNV-1a, 64 bits (the generator's digest).
pub(crate) fn fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in data {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The inputs per digest chunk.
pub(crate) const CHUNK: usize = 64;

/// Compares `outputs` (one per input) with the generator's per-chunk
/// digests; `Err` names the inputs of the first chunk that differs.
pub(crate) fn compare_digests(
    what: &str,
    inputs: &[&str],
    outputs: &[Vec<u8>],
    digests: &str,
) -> Result<(), String> {
    let want: Vec<&str> = digests.split(',').collect();
    let chunks: Vec<&[Vec<u8>]> = outputs.chunks(CHUNK).collect();
    if chunks.len() != want.len() {
        return Err(format!(
            "{what}: {} chunks, the table has {}",
            chunks.len(),
            want.len()
        ));
    }
    for (k, (chunk, want)) in chunks.iter().zip(want).enumerate() {
        let mut data = Vec::new();
        for o in *chunk {
            data.extend_from_slice(o);
            data.push(0x1e);
        }
        if format!("{:016x}", fnv(&data)) != want {
            let lines: Vec<String> = (k * CHUNK..k * CHUNK + chunk.len())
                .map(|i| {
                    format!(
                        "{:?} -> {:?}",
                        inputs[i],
                        String::from_utf8_lossy(&outputs[i])
                    )
                })
                .collect();
            return Err(format!(
                "{what}: chunk {k} differs from the tool (run the script's code on these lines to see its output); ours:\n{}",
                lines.join("\n")
            ));
        }
    }
    Ok(())
}

/// Proves `f` equal to the script code the generator ran as snippet
/// `name` (rows `HD` of the corpus): `f` gets each input line and returns
/// what the code printed. Returns the number of inputs compared.
pub fn check_snippet(name: &str, f: impl Fn(&str) -> Vec<u8>) -> Result<usize, String> {
    let input = inputs();
    let row = rows(CORPUS)
        .find(|r| r[0] == "HD" && r[1] == name)
        .ok_or_else(|| format!("no snippet {name:?} in data/family002/corpus.tsv"))?;
    let subset: Vec<&str> = ids(row[2]).into_iter().map(|i| input[i].as_str()).collect();
    let outputs: Vec<Vec<u8>> = subset.iter().map(|l| f(l)).collect();
    compare_digests(name, &subset, &outputs, row[3])?;
    Ok(subset.len())
}
