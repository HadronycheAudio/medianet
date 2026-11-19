//! `wire.toml` as the document checks read it.
//!
//! The file is parsed with the crate's existing TOML dependency (the same
//! parser [`crate::wire_tables`] uses), not with the subset readers the
//! scripts carried. The checks read raw keys (`meaning`, `valid`, `width`,
//! `exempt_from_control_bound` ...) that the typed model of
//! [`crate::wire_tables`] does not keep, so this is a thin view over
//! [`toml::Table`].

use std::collections::BTreeMap;

use toml::{Table, Value};

use super::docs::Docs;

/// Where the machine-readable tables live.
pub const WIRE_TOML: &str = "doc/design/spec/wire.toml";

/// A parsed `wire.toml`.
#[derive(Debug, Clone)]
pub struct WireToml {
    pub root: Table,
}

impl WireToml {
    /// Parses `text`.
    pub fn parse(text: &str) -> Result<WireToml, String> {
        text.parse::<Table>()
            .map(|root| WireToml { root })
            .map_err(|e| e.to_string())
    }

    /// Reads and parses the file of `docs`.
    pub fn load(docs: &Docs) -> Result<WireToml, String> {
        let text = docs
            .read(WIRE_TOML)
            .ok_or_else(|| format!("{WIRE_TOML} does not exist"))?;
        WireToml::parse(&text)
    }

    /// The tables of the array `key` (`message`, `reserved`, `refusal`).
    pub fn array(&self, key: &str) -> Vec<&Table> {
        tables_of(self.root.get(key))
    }

    /// Every `[[message]]`.
    pub fn messages(&self) -> Vec<&Table> {
        self.array("message")
    }

    /// The `[[message]]` named `name`.
    pub fn message(&self, name: &str) -> Option<&Table> {
        self.messages()
            .into_iter()
            .find(|m| text(m, "name") == name)
    }

    /// Every field entry by name: the `[[message.field]]` entries of every
    /// message, then the `[[reserved]]` entries (004-22's `FIELD_INFO`).
    pub fn field_info(&self) -> BTreeMap<String, Vec<&Table>> {
        let mut out: BTreeMap<String, Vec<&Table>> = BTreeMap::new();
        for m in self.messages() {
            for f in fields(m) {
                out.entry(text(f, "name").to_string()).or_default().push(f);
            }
        }
        for r in self.array("reserved") {
            out.entry(text(r, "name").to_string()).or_default().push(r);
        }
        out
    }
}

/// The tables of an array value.
pub fn tables_of(value: Option<&Value>) -> Vec<&Table> {
    value
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_table).collect())
        .unwrap_or_default()
}

/// The `[[message.field]]` entries of a message.
pub fn fields(message: &Table) -> Vec<&Table> {
    tables_of(message.get("field"))
}

/// A string key, or `""`.
pub fn text<'a>(table: &'a Table, key: &str) -> &'a str {
    table.get(key).and_then(Value::as_str).unwrap_or("")
}

/// An integer key.
pub fn int(table: &Table, key: &str) -> Option<i64> {
    table.get(key).and_then(Value::as_integer)
}

/// A value as Python's `str()` would print it (strings bare).
pub fn display(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Boolean(true) => "True".into(),
        Value::Boolean(false) => "False".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_real_file_reads() {
        let w =
            WireToml::load(&crate::doccheck::testing::repo().clone()).expect("wire.toml parses");
        let media = w.message("media packet").expect("media packet");
        assert!(!fields(media).is_empty());
        let info = w.field_info();
        assert!(info.contains_key("version"));
        assert!(info["version"].iter().any(|f| int(f, "width").is_some()));
    }

    #[test]
    fn accessors_on_a_small_file() {
        let w = WireToml::parse(
            "[[message]]\nname = \"m\"\nmax_size = 4\n[[message.field]]\nname = \"a\"\nwidth = 32\n[[reserved]]\nname = \"r\"\n",
        )
        .unwrap();
        let m = w.message("m").unwrap();
        assert_eq!(int(m, "max_size"), Some(4));
        assert_eq!(text(fields(m)[0], "name"), "a");
        assert_eq!(w.field_info().len(), 2);
        assert_eq!(display(&Value::Boolean(true)), "True");
        assert!(WireToml::parse("[[").is_err());
        assert!(w.message("x").is_none());
    }
}
