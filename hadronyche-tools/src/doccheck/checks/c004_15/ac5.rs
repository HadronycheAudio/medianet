//! AC 5 (optional toml keys): any integer key naming a wire size, rate or
//! overhead agrees with the computed value.

use std::collections::BTreeSet;

use super::num::Fails;
use super::setup::Calc;
use crate::doccheck::pynum::record_int_limit;
use crate::doccheck::text::rx;
use crate::doccheck::tomlline::{Table, Value};

fn key_tokens(name: &str) -> BTreeSet<String> {
    rx(r"[^a-z0-9]+")
        .split(&name.to_lowercase())
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// What the key must equal, and how it is called: by the tokens of the key
/// and of its table name.
fn wanted(toks: &BTreeSet<String>, k: &Calc) -> Option<(Vec<i128>, String)> {
    let has = |w: &str| toks.contains(w);
    let any = |ws: &[&str]| ws.iter().any(|w| toks.contains(*w));
    if has("overhead") && !any(&["ip", "udp", "ethernet", "ipv4"]) {
        Some((vec![k.ovh], format!("the overhead of {} bytes", k.ovh)))
    } else if has("ethernet") && any(&["overhead", "framing"]) && !any(&["ip", "udp"]) {
        Some((
            vec![k.eth],
            format!("the Ethernet framing of {} bytes", k.eth),
        ))
    } else if any(&["ipv4", "ip"]) && has("header") {
        Some((vec![20], "the IPv4 header of 20 bytes".into()))
    } else if has("udp") && has("header") {
        Some((vec![8], "the UDP header of 8 bytes".into()))
    } else if has("wire")
        && has("default")
        && !any(&["rate", "bit", "stream", "channel", "kbit", "mbit", "bps"])
    {
        Some((
            vec![k.wire],
            format!("the default on-wire size of {} bytes", k.wire),
        ))
    } else if has("wire") && any(&["worst", "maximum", "max"]) && !any(&["rate", "bit", "stream"]) {
        Some((
            vec![k.wire_max],
            format!("the format maximum on-wire size of {} bytes", k.wire_max),
        ))
    } else if has("rate") && has("default") && has("stream") {
        let kbit = (k.bits_stream as f64 / 1000.0).round_ties_even() as i128;
        Some((
            vec![k.bits_stream, kbit],
            format!("the default per-stream rate of {} bit/s", k.bits_stream),
        ))
    } else {
        None
    }
}

/// The toml keys.
pub(crate) fn toml_keys(k: &Calc, f: &mut Fails) {
    for (tname, value) in &k.doc.others.0 {
        let blocks: Vec<&Table> = match value {
            Value::List(xs) => xs.iter().filter_map(Value::as_table).collect(),
            Value::Dict(t) => vec![t],
            _ => Vec::new(),
        };
        for d in blocks {
            for (key, v) in &d.0 {
                let n = match v {
                    Value::Int(n) => *n,
                    Value::BigInt(s) => {
                        record_int_limit(s);
                        continue;
                    }
                    _ => continue,
                };
                let toks: BTreeSet<String> =
                    key_tokens(key).union(&key_tokens(tname)).cloned().collect();
                if !["wire", "rate", "overhead"]
                    .iter()
                    .any(|w| toks.contains(*w))
                {
                    continue;
                }
                if let Some((want, label)) = wanted(&toks, k)
                    && !want.contains(&n)
                {
                    f.fail("AC5", format!("wire.toml [{tname}] key {key:?} = {n}, but {label} is computed from the layout"));
                }
            }
        }
    }
}
