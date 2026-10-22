//! `assets/data/sounds.json`: every `BCSound` def with its audio format, plus the runtime
//! sound aliases (`docs/re/audio-prefs-text.md` §1.10). Same defs and order as
//! `sound-defs.json`, with what the audio layer needs on top.

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/sounds.json";

/// Deserializes the embedded `sounds.json`.
pub fn load() -> Result<SoundTable, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SoundTable {
    pub format_version: u32,

    /// Struct-XML (parse) order.
    pub sounds: Vec<SoundDef>,

    pub aliases: Vec<SoundAlias>,
}

/// One sound def.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SoundDef {
    pub id: String,

    /// Base name in the bundle's `sound/` folder; `None` for `Vibrate`.
    pub file_name: Option<String>,

    /// `sounds/<file_name>.flac`, relative to `assets/`.
    pub asset: Option<String>,

    /// `BCSound.mbLoop`.
    pub looping: bool,

    /// A new voice per play.
    pub multiple_instances: bool,

    /// Always false in 1.48.2.
    pub stream: bool,

    /// True only for `Vibrate`.
    pub vibrate: bool,

    /// 22000 or 44100.
    pub sample_rate: Option<u32>,

    /// 1 or 2.
    pub channels: Option<u8>,

    /// Sample frames (duration = frames / sample rate).
    pub frames: Option<u32>,
}

/// A runtime alias (`setSoundAlias:sound:`) and every sound it can point at.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SoundAlias {
    pub alias: String,

    /// The code or data that sets it.
    pub set_by: String,

    /// Sound ids, sorted.
    pub targets: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sound_defs;

    #[test]
    fn agrees_with_the_struct_xml_sound_defs() {
        let table = load().expect("sounds.json matches its schema");
        let defs = sound_defs::load().expect("sound-defs.json matches its schema");
        assert_eq!(table.sounds.len(), 359);
        assert_eq!(table.sounds.len(), defs.sounds.len());

        for (sound, def) in table.sounds.iter().zip(&defs.sounds) {
            assert_eq!(sound.id, def.id);
            assert_eq!(sound.looping, def.looping, "{}", sound.id);
            assert_eq!(sound.multiple_instances, def.multiple_instances, "{}", sound.id);
            assert_eq!(sound.vibrate, sound.id == "Vibrate");
            if let Some(asset) = &sound.asset {
                assert!(crate::file(asset).is_some(), "{asset} is not embedded");
            }
        }
        assert!(table.aliases.iter().all(|alias| alias.alias == "BackgroundLoop"));
    }
}
