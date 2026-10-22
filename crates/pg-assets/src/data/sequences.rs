//! `assets/data/sequences.json`: every `BCSequenceDef` of the list at 0x100643a60, in list
//! order (`docs/re/sequences.md` §2.1).

use std::fmt;

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};

use super::DataError;

pub const PATH: &str = "data/sequences.json";

/// Deserializes the embedded `sequences.json`.
pub fn load() -> Result<Vec<SequenceDef>, DataError> {
    super::load(PATH)
}

/// One sequence (`BCSequenceDef`).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SequenceDef {
    pub id: String,

    /// Items in execution order (may be empty).
    pub items: Vec<SequenceItem>,
}

/// One item (`BCSequenceItemDef`); the selector is `method + ":::"`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SequenceItem {
    pub method: String,
    pub params: Params,
}

/// The parameters present in an item, keyed by schema name, in positional order (the order
/// of the JSON object). Absent parameters are omitted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Params(pub Vec<(String, String)>);

impl Params {
    /// The value of the parameter `name`, `None` when absent.
    pub fn get(&self, name: &str) -> Option<&str> {
        let entry = self.0.iter().find(|(key, _)| key == name)?;
        Some(entry.1.as_str())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

struct ParamsVisitor;

impl<'de> Visitor<'de> for ParamsVisitor {
    type Value = Params;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object of string parameters")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Params, A::Error> {
        let mut entries = Vec::new();
        while let Some(entry) = map.next_entry::<String, String>()? {
            entries.push(entry);
        }
        Ok(Params(entries))
    }
}

impl<'de> Deserialize<'de> for Params {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Params, D::Error> {
        deserializer.deserialize_map(ParamsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::data::sequence_methods;

    #[test]
    fn deserializes_with_documented_counts() {
        let sequences = load().expect("sequences.json matches its schema");
        assert_eq!(sequences.len(), 2548);

        let item_count: usize = sequences.iter().map(|sequence| sequence.items.len()).sum();
        assert_eq!(item_count, 7258);

        let ids: HashSet<&str> = sequences.iter().map(|sequence| sequence.id.as_str()).collect();
        assert_eq!(ids.len(), sequences.len(), "sequence ids are unique");
    }

    #[test]
    fn params_keep_document_order() {
        let json = r#"{"method": "setPos", "params": {"zPos": "1", "xPos": "2", "yPos": "3"}}"#;
        let item: SequenceItem = serde_json::from_str(json).expect("valid item");
        let keys: Vec<&str> = item.params.0.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["zPos", "xPos", "yPos"]);
        assert_eq!(item.params.get("xPos"), Some("2"));
        assert_eq!(item.params.get("id"), None);
    }

    #[test]
    fn params_are_in_positional_order() {
        let sequences = load().expect("sequences.json matches its schema");
        let methods = sequence_methods::load().expect("sequence-methods.json matches its schema");

        let mut positions: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
        for method in &methods {
            let mut by_name = HashMap::new();
            for param in &method.params {
                by_name.insert(param.name.as_str(), param.index);
            }
            positions.insert(method.method.as_str(), by_name);
        }

        for sequence in &sequences {
            for item in &sequence.items {
                let Some(by_name) = positions.get(item.method.as_str()) else {
                    panic!("{}: method {} has no schema", sequence.id, item.method);
                };
                let mut previous = None;
                for (name, _) in &item.params.0 {
                    let Some(&index) = by_name.get(name.as_str()) else {
                        panic!("{}: {} has no parameter {name}", sequence.id, item.method);
                    };
                    if let Some(previous) = previous {
                        assert!(index > previous, "{}: {} parameters out of order", sequence.id, item.method);
                    }
                    previous = Some(index);
                }
            }
        }
    }
}
