//! Typed views of the JSON tables in `assets/data/`, extracted from the original executable
//! by `tools/re/`. Every type mirrors its file exactly (key names, nesting, nullability); the
//! schemas are documented in `docs/re/textures.md` §9, `docs/re/struct-xml.md` §13,
//! `docs/re/sequences.md` §2, `docs/re/logic.md` §13.4 and `docs/re/audio-prefs-text.md`.
//!
//! Each module has the path of its file and a `load()` that deserializes the embedded copy.
//! `sounds`, `fonts` and `gradients` overlap with the struct-XML tables `sound_defs`,
//! `font_defs` and `gradient_defs` (same nodes, same values) and add audio formats, aliases
//! and glyph metrics.

pub mod dance_beats;
pub mod debug_items;
pub mod display_groups;
pub mod episodes;
pub mod font_defs;
pub mod fonts;
pub mod gradient_defs;
pub mod gradients;
pub mod konkey_dong_levels;
pub mod links;
pub mod logic_objects;
pub mod preferences;
pub mod sequence_methods;
pub mod sequences;
pub mod skin_packs;
pub mod sound_defs;
pub mod sounds;
pub mod textures;

use std::fmt;

use serde::de::DeserializeOwned;

/// Why a data table could not be loaded.
#[derive(Debug)]
pub enum DataError {
    /// The file is not among the embedded assets.
    Missing { path: &'static str },

    /// The file does not match its schema.
    Json { path: &'static str, source: serde_json::Error },
}

impl fmt::Display for DataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Missing { path } => write!(formatter, "assets/{path} is not embedded"),
            DataError::Json { path, source } => write!(formatter, "assets/{path}: {source}"),
        }
    }
}

impl std::error::Error for DataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataError::Missing { .. } => None,
            DataError::Json { source, .. } => Some(source),
        }
    }
}

/// Deserializes the embedded JSON file at `path` (relative to `assets/`).
pub fn load<T: DeserializeOwned>(path: &'static str) -> Result<T, DataError> {
    let Some(bytes) = crate::file(path) else {
        return Err(DataError::Missing { path });
    };

    match serde_json::from_slice(bytes) {
        Ok(value) => Ok(value),
        Err(source) => Err(DataError::Json { path, source }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_files_are_reported() {
        let result = load::<serde_json::Value>("data/no-such-file.json");
        assert!(matches!(result, Err(DataError::Missing { .. })));
    }

    /// The schemas are strict under test, so a key the types do not model fails the
    /// deserialization tests of every table.
    #[test]
    fn unknown_keys_are_rejected_under_test() {
        let json = r#"{"items": ["ResetChallenges"], "unexpected": 1}"#;
        let result = serde_json::from_str::<debug_items::DebugItemsFile>(json);
        assert!(result.is_err());

        let step = r#"{"op": "set_behavior", "sequence_id": "Init", "unexpected": 1}"#;
        let result = serde_json::from_str::<logic_objects::InitStep>(step);
        assert!(result.is_err());
    }

    #[test]
    fn every_data_file_is_modelled() {
        let mut modelled = vec![
            dance_beats::PATH,
            debug_items::PATH,
            display_groups::PATH,
            episodes::PATH,
            font_defs::PATH,
            fonts::PATH,
            gradient_defs::PATH,
            gradients::PATH,
            konkey_dong_levels::PATH,
            links::PATH,
            logic_objects::PATH,
            preferences::PATH,
            sequence_methods::PATH,
            sequences::PATH,
            skin_packs::PATH,
            sound_defs::PATH,
            sounds::PATH,
            textures::PATH,
        ];
        modelled.sort_unstable();

        let mut embedded = Vec::new();
        for file in crate::files_in("data") {
            if file.path.ends_with(".json") {
                embedded.push(file.path);
            }
        }
        assert_eq!(embedded, modelled);
    }
}
