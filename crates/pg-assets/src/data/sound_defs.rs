//! `assets/data/sound-defs.json`: the `<sound>` nodes of `parseStructXml:` call 1
//! (root 0x10071ab98), in parse order (`docs/re/struct-xml.md` §4.1, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/sound-defs.json";

/// Deserializes the embedded `sound-defs.json`.
pub fn load() -> Result<SoundDefsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SoundDefsFile {
    pub sounds: Vec<SoundDef>,
}

/// One sound node (type 0): the arguments of
/// `initWithId:fileName:ext:loop:stream:multipleInstances:preload:NO`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SoundDef {
    pub id: String,

    /// Resource name: the sample is `sound/<file_name>.<ext>` (empty for `Vibrate`).
    pub file_name: String,

    /// Resource extension (`caf`; empty for `Vibrate`).
    pub ext: String,

    #[serde(rename = "loop")]
    pub looping: bool,

    pub multiple_instances: bool,
    pub stream: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("sound-defs.json matches its schema");
        assert_eq!(file.sounds.len(), 359);

        let looping_count = file.sounds.iter().filter(|sound| sound.looping).count();
        assert_eq!(looping_count, 39);

        let multiple_count = file.sounds.iter().filter(|sound| sound.multiple_instances).count();
        assert_eq!(multiple_count, 18);
        assert!(file.sounds.iter().all(|sound| !sound.stream));
    }
}
