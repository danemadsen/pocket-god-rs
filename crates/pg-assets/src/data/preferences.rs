//! `assets/data/preferences.json`: the four `BCPreferences` documents of `loadGame`, the
//! defaults registered by `loadPreferences:::`, the personas and their per-persona key formats
//! (`docs/re/audio-prefs-text.md` §2.8).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/preferences.json";

/// Deserializes the embedded `preferences.json`.
pub fn load() -> Result<PreferencesFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PreferencesFile {
    pub format_version: u32,
    pub documents: Vec<PreferenceDocument>,

    /// In `loadPreferences:::` call order.
    pub defaults: Vec<PreferenceDefault>,

    pub personas: Vec<PersonaDef>,
    pub persona_keys: Vec<PersonaKey>,
}

/// One `BCPreferences` document (`initWithId:secure:iCloud:`).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PreferenceDocument {
    /// `Persona`, `Options`, `System` or `Challenge`.
    pub id: String,

    /// The view controller ivar holding it.
    pub ivar: String,

    pub file_name: String,
    pub icloud_file_name: String,

    /// `setLegacyPreferenceFileName:` (`Preferences.plist`), if any.
    pub legacy_file_name: Option<String>,

    pub secure: bool,
    pub icloud: bool,
}

/// The type of a preference value (`setDefaultPreference:as…:`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreferenceValueType {
    Int,
    Float,
    String,
    Dictionary,
}

/// One `setDefaultPreference:asInt:` call.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PreferenceDefault {
    pub document: String,
    pub key: String,
    pub value_type: PreferenceValueType,
    pub value: i32,

    /// Address of the call.
    pub address: String,
}

/// What a persona is in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaRole {
    Main,
    Pygmy,
    GuestPygmy,
}

/// One persona created by `loadPreferences:::`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PersonaDef {
    pub id: String,

    /// `GodPersona`, `TribePersona` or `PygmyPersona`.
    pub class: String,

    pub default_name: String,
    pub default_description: String,

    /// The document holding its keys (`Persona`).
    pub document: String,

    pub personality: Option<i32>,
    pub role: PersonaRole,
}

/// Which persona field supplies a key's default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaDefaultField {
    DefaultName,
    DefaultDescription,
}

/// A per-persona preference key set up by a persona initializer.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct PersonaKey {
    /// `Persona` (every persona) or `GodPersona` (the `God` persona only).
    pub class: String,

    /// `"%@"` stands for the persona id.
    pub key_format: String,

    pub value_type: PreferenceValueType,
    pub default_value: Option<i32>,
    pub default_from: Option<PersonaDefaultField>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("preferences.json matches its schema");
        let ids: Vec<&str> = file.documents.iter().map(|document| document.id.as_str()).collect();
        assert_eq!(ids, ["Persona", "Options", "System", "Challenge"]);
        assert_eq!(file.defaults.len(), 48);
        assert_eq!(file.personas.len(), 10);
        assert_eq!(file.persona_keys.len(), 9);
        assert!(file.defaults.iter().all(|default| default.value_type == PreferenceValueType::Int));
    }
}
