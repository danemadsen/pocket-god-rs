//! `assets/data/debug-items.json`: the debug screen entry names (type 36, list 0x100730938)
//! (`docs/re/struct-xml.md` §11, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/debug-items.json";

/// Deserializes the embedded `debug-items.json`.
pub fn load() -> Result<DebugItemsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DebugItemsFile {
    pub items: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_items() {
        let file = load().expect("debug-items.json matches its schema");
        assert_eq!(file.items.len(), 11);
        assert_eq!(file.items[0], "ResetChallenges");
        assert_eq!(file.items[10], "RemovePrefs");
    }
}
