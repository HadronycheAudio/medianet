//! The Manifest section and its field tables: AC1 (every Context item has
//! a field with a width and a unit) and the review points on the fields
//! (point 4, the stream element's channel count in wire.toml; T3 (a), the
//! L range against the landed hop count and saturation; T3 (f), the audio
//! length bound).

use super::toml_side::Toml;
use super::{MRow, NUM, Wire, cells, is_field_table, is_validity_table, model_rows, tile, to_int};
use crate::doccheck::checked;
use crate::doccheck::Report;
use crate::doccheck::report::Flow;
use crate::doccheck::family004::tables;
use crate::doccheck::sentences::{RAW_COLON, split_raw};
use crate::doccheck::text::{has, has_i, rx};
use crate::doccheck::units::{BlockKind, prose_blocks, strip_fences};

/// One field table: rows, bit offsets, total bits, where.
pub(crate) struct Model {
    pub rows: Vec<MRow>,
    pub offs: Vec<i128>,
    pub total: i128,
    pub place: String,
}

/// The Manifest section as the criteria read it.
pub(crate) struct Man {
    pub lines: Vec<String>,
    pub text: String,
    /// `FLAT`: fences blanked, white space squashed.
    pub flat: String,
    pub models: Vec<Model>,
    /// `FORMS`.
    pub forms: Vec<(String, String)>,
    /// `PARAS_FLAT`.
    pub paras_flat: Vec<String>,
}

impl Man {
    /// `ALL_ROWS`.
    pub fn all_rows(&self) -> impl Iterator<Item = &MRow> {
        self.models.iter().flat_map(|m| m.rows.iter())
    }

    /// `rows_where(pred)` is not empty.
    pub fn any_row(&self, pred: impl Fn(&MRow) -> bool) -> bool {
        self.all_rows().any(pred)
    }

    /// The sentences of `FLAT` (`re.split(r"(?<=[.;:])\s+", FLAT)`).
    pub fn flat_sentences(&self) -> Vec<String> {
        split_raw(&self.flat, RAW_COLON).into_iter().map(String::from).collect()
    }

    /// The sentences of `" ".join(PARAS_FLAT)`.
    pub fn para_sentences(&self) -> Vec<String> {
        split_raw(&self.paras_flat.join(" "), RAW_COLON).into_iter().map(String::from).collect()
    }
}

/// Reads the Manifest section and checks its tables.
pub(crate) fn manifest(wire: &Wire, report: &mut Report) -> Man {
    let lines = wire.section_lines(r"^Manifest$", 2);
    report.check("AC1", lines.is_some(), "wire.md has a '## Manifest' section");
    let lines = lines.unwrap_or_default();
    let text = lines.join("\n");
    let flat = squash_keep(&strip_fences(&text));
    let all = tables(&text);
    let ftabs: Vec<_> = all.iter().filter(|t| is_field_table(t.head())).collect();
    report.check("AC1", !ftabs.is_empty(), format!("the Manifest section has field tables ({} found)", ftabs.len()));
    let mut models = Vec::new();
    for (k, t) in ftabs.iter().enumerate() {
        let (rows, bad) = model_rows(&cells(t));
        let place = format!("Manifest field table {}", k + 1);
        for b in bad {
            report.fail("AC1", format!("{place}: {b}"));
        }
        let (offs, total, errs) = tile(&rows);
        for e in errs {
            report.fail("AC1", format!("{place}: {e}"));
        }
        models.push(Model { rows, offs, total, place });
    }
    let forms = all
        .iter()
        .filter(|t| is_validity_table(t.head()) && !is_field_table(t.head()))
        .flat_map(|t| t.rows.iter().filter(|r| r.cells.len() >= 2).map(|r| (r.cell(0).to_string(), r.cell(1).to_string())))
        .collect();
    let prose = prose_blocks(&text);
    let paras_flat = prose.iter().filter(|b| b.kind == BlockKind::Prose).map(|b| squash_keep(&strip_fences(&b.text))).collect();
    Man {
        lines,
        text,
        flat,
        models,
        forms,
        paras_flat,
    }
}

/// `re.sub(r"\s+", " ", t)` (no strip).
fn squash_keep(t: &str) -> String {
    rx(r"\s+").replace_all(t, " ")
}

const TIME_UNIT: &str = r"(?:µs|μs|\bus\b|microseconds?|\bns\b|nanoseconds?|\bms\b|milliseconds?)";

fn any_unit() -> String {
    format!(r"(?:samples?|frames?|operations?|bits?|bytes?|Hz|seconds?|\bs\b|{TIME_UNIT}|\bns\b)")
}

/// `REQ_E` / `DEC_E`: rows naming `word` and `\bE\b`.
pub(crate) fn e_rows<'a>(man: &'a Man, word: &str) -> Vec<&'a MRow> {
    man.all_rows().filter(|r| r.txt().contains(word) && has(r"\bE\b", &format!("{} {}", r.meaning, r.valid))).collect()
}

/// AC1.
pub(crate) fn ac1(man: &Man, report: &mut Report) {
    let ports = man.any_row(|r| r.meaning.to_lowercase().contains("port") || r.valid.to_lowercase().contains("port"));
    report.check("AC1", ports && man.any_row(|r| r.txt().contains("content type")), "ports and content types have fields");
    report.check("AC1", man.any_row(|r| r.txt().contains("channel map")), "the channel map has a field");
    let l = man.any_row(|r| (has(r"\bL\b", &r.meaning) || r.meaning.to_lowercase().contains("latency")) && r.txt().contains("sample"));
    report.check("AC1", l, "L has a field whose valid range is in whole samples (unit stated)");
    let join: Vec<&MRow> = man.all_rows().filter(|r| r.txt().contains("join")).collect();
    report.check("AC1", join.iter().any(|r| r.maxc.is_some()), "the join-delay count is a counted row with a fixed maximum");
    report.check("AC1", join.iter().any(|r| r.txt().contains("sample")), "each inserted join delay is a whole number of samples");
    report.check("AC1", join.iter().any(|r| r.txt().contains("frequency")), "the frequency-dependent-delay flag of each join (C-104) has a field");
    report.check("AC1", man.any_row(|r| r.txt().contains("initial value")), "dense controller initial values (C-158) have a field");
    let conv: Vec<&MRow> = man.all_rows().filter(|r| r.txt().contains("conversion delay") && r.txt().contains("device")).collect();
    report.check("AC1", !conv.is_empty(), "the conversion delay on a device-facing port (C-207) has a field");
    let unit = any_unit();
    report.check("AC1", conv.iter().any(|r| has_i(&unit, &format!("{} {} {}", r.meaning, r.valid, r.wcell))), "the conversion delay states its unit");
    let cost: Vec<&MRow> = man.all_rows().filter(|r| r.txt().contains("cost bound") && r.txt().contains("operation")).collect();
    report.check("AC1", !cost.is_empty(), "the cost bound (C-106) has a field in operations");
    report.check("AC1", cost.iter().any(|r| r.txt().contains("slice")), "the cost bound is per slice (C-106)");
    let (req, dec) = (e_rows(man, "required"), e_rows(man, "declared"));
    report.check("AC1", !req.is_empty(), "required E per presentation group has a field");
    report.check("AC1", !dec.is_empty(), "declared E per sink has a field");
    report.check("AC1", req.iter().chain(dec.iter()).any(|r| has_i(TIME_UNIT, &r.txt())), "the unit of E is stated");
}

/// `(numeric_range)`: the first `lo to hi` of a valid range.
pub(crate) fn numeric_range(text: &str) -> Option<(i128, i128)> {
    let n = format!("({}|{NUM})", super::HEXNUM);
    let c = rx(&format!(r"{n}\s*(?:to|\.\.|–|—|-)\s*{n}")).captures(text)?;
    Some((to_int(&c[1])?, to_int(&c[2])?))
}

fn nums(text: &str) -> Vec<i128> {
    rx(NUM).find_iter(text).filter_map(|m| to_int(m.as_str())).collect()
}

/// Review point 4; `Err` where the script stops.
pub(crate) fn point4(man: &Man, toml: &Toml, report: &mut Report) -> Flow<()> {
    let mut loc = None;
    for (k, m) in man.models.iter().enumerate() {
        for (i, r) in m.rows.iter().enumerate() {
            if r.meaning.to_lowercase().starts_with("channel count: how many channel-map entries") {
                loc = Some((k, m.offs[i]));
            }
        }
    }
    let Some((k, boff)) = loc else {
        report.fail("AC1", "the markdown stream element has a channel-count row");
        return Ok(());
    };
    let m = &man.models[k];
    let sig: Vec<(i128, i128, Option<i128>)> = m.rows.iter().zip(&m.offs).map(|(r, o)| (*o, r.bits, r.maxc)).collect();
    let texts = toml.texts_at(&sig, boff)?;
    report.check("AC1", texts.iter().any(|t| has("65[ \u{a0}\u{202f}]?535", t)), "wire.toml's channel-count field of the stream element states the audio-length bound (65 535)");
    Ok(())
}

/// Review T3 (a): the L range against the landed hop count and saturation.
pub(crate) fn t3a(man: &Man, wire: &Wire, report: &mut Report) {
    let l_rows: Vec<&MRow> = man.all_rows().filter(|r| r.meaning.to_lowercase().starts_with("structural latency")).collect();
    let hop = rx(&format!("maximum hop count of ({NUM})")).captures(&wire.raw).and_then(|c| to_int(&c[1]));
    let sat = rx(&format!("Saturates at ({NUM}) samples")).captures(&wire.raw).and_then(|c| to_int(&c[1]));
    let l_hi = l_rows.first().and_then(|r| numeric_range(&r.valid)).map(|r| r.1);
    match (l_hi, hop, sat) {
        (Some(l), Some(h), Some(s)) => report.check("AC1", checked::mul(h, l) <= s, format!("the largest L ({l} samples) times the landed maximum hop count ({h}) is within the landed declared-delay saturation value {s}, so the sum cannot saturate (Measurement extension, Open item)")),
        _ => report.check("AC1", false, "the L range, the landed maximum hop count and the landed saturation value are all readable"),
    };
}

/// Review T3 (f): the audio length bound.
pub(crate) fn t3f(man: &Man, wire: &Wire, report: &mut Report) {
    let aud = rx(&format!(r"frames × channels × 4, the samples that follow \|[^\n]*?at most ({NUM})")).captures(&wire.raw).and_then(|c| to_int(&c[1]));
    let stated = aud.is_some_and(|a| {
        man.flat_sentences().iter().any(|s| has_i("channel", s) && nums(s).contains(&a))
            || man.forms.iter().any(|(f, o)| nums(&format!("{f} {o}")).contains(&a) && has_i("channel|frames", f))
    });
    report.check("AC1", stated, format!("the manifest states the landed audio length bound ({aud:?}) for frames × channels × 4, or an invalid form for exceeding it"));
}
